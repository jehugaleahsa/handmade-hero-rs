mod application;
mod direct_sound;
mod direct_sound_buffer;
mod direct_sound_buffer_lock_guard;
mod pixel;

use crate::application::Application;
use crate::direct_sound::{
    DirectSound, BYTES_PER_SAMPLE, SOUND_BUFFER_SIZE, SQUARE_WAVE_MID_PERIOD, VOLUME,
};
use crate::direct_sound_buffer::DirectSoundBuffer;
use std::ffi::c_void;
use windows::core::{w, Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_SUCCESS, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE, XUSER_MAX_COUNT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetWindowLongPtrW, LoadCursorW, PeekMessageW,
    RegisterClassW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    GWL_USERDATA, IDC_ARROW, MSG, PM_REMOVE, WINDOW_EX_STYLE,
    WM_NCCREATE, WM_QUIT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn main() -> Result<()> {
    let instance = get_instance()?;
    let class_name = create_window_class(instance)?;

    let mut application = Application::new();
    let window_handle = create_window(instance, class_name, &mut application)?;

    application.resize_buffer(720, 480).unwrap_or(LRESULT(0));

    let direct_sound = DirectSound::initialize(window_handle).ok();
    let sound_buffer = direct_sound
        .as_ref()
        .and_then(|ds| ds.create_buffer(SOUND_BUFFER_SIZE).ok());
    if let Some(ref buffer) = sound_buffer {
        buffer.play_looping().unwrap_or(());
    }

    run_application_loop(
        &mut application,
        window_handle,
        sound_buffer.as_ref(),
        SOUND_BUFFER_SIZE,
        VOLUME,
    )?;

    Ok(())
}

fn get_instance() -> Result<HINSTANCE> {
    let instance = unsafe { GetModuleHandleW(None)? };
    Ok(instance.into())
}

fn create_window_class(instance: HINSTANCE) -> Result<PCWSTR> {
    let class_name = w!("Handmade Hero");
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };
    let window_class = WNDCLASSW {
        hCursor: cursor,
        hInstance: instance,
        lpszClassName: class_name,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_procedure),
        ..Default::default()
    };

    let register_result = unsafe { RegisterClassW(&raw const window_class) };
    if register_result == 0 {
        let error = unsafe { GetLastError() };
        return Err(Error::from(error));
    }
    Ok(class_name)
}

fn create_window(
    instance: HINSTANCE,
    class_name: PCWSTR,
    application: &mut Application,
) -> Result<HWND> {
    let application_pointer = std::ptr::from_ref::<Application>(application).cast::<c_void>();
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Handmade Hero"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            600,
            None,
            None,
            Some(instance),
            Some(application_pointer),
        )
    }
}

extern "system" fn window_procedure(
    window_handle: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create_struct = unsafe { &*(l_param.0 as *const CREATESTRUCTW) };
        let application = create_struct.lpCreateParams.cast::<Application>();
        unsafe { SetWindowLongPtrW(window_handle, GWL_USERDATA, application as isize) };
        return LRESULT(1); // Indicate we should proceed with creating the window.
    }

    let application_pointer =
        unsafe { GetWindowLongPtrW(window_handle, GWL_USERDATA) } as *mut Application;
    if application_pointer.is_null() {
        // We're not initialized yet, so just let the default handler run.
        return unsafe { DefWindowProcW(window_handle, message, w_param, l_param) };
    }

    // We keep an Application object alive for the duration of the application.
    // This allows us to maintain state about the application without relying on
    // global variables.
    let application = unsafe { &mut *application_pointer };
    application.process_windows_message(window_handle, message, w_param, l_param)
}

fn run_application_loop(
    application: &mut Application,
    window_handle: HWND,
    sound_buffer: Option<&DirectSoundBuffer<'_>>,
    sound_buffer_size: u32,
    volume: i16,
) -> Result<()> {
    let mut running_sample_index = 0u32;
    loop {
        let mut message = MSG::default();
        let message_result = unsafe { PeekMessageW(&raw mut message, None, 0, 0, PM_REMOVE) };
        if message_result.0 < 0 || message.message == WM_QUIT {
            return Ok(());
        }
        unsafe {
            #[allow(unused_must_use)]
            TranslateMessage(&raw const message);
            DispatchMessageW(&raw const message);
        };

        if let Some(controller_state) = poll_controller_state() {
            let gamepad = &controller_state.Gamepad;
            application.shift_x(gamepad.sThumbRX);
            application.shift_y(gamepad.sThumbRY);
        } else {
            application.shift_x(1);
            application.shift_y(1);
        }

        application.update_display(window_handle)?;

        running_sample_index = play_sound(
            sound_buffer,
            sound_buffer_size,
            running_sample_index,
            volume,
        );
    }
}

fn play_sound(
    sound_buffer: Option<&DirectSoundBuffer<'_>>,
    sound_buffer_size: u32,
    running_sample_index: u32,
    volume: i16,
) -> u32 {
    let Some(sound_buffer) = sound_buffer else {
        return running_sample_index;
    };
    let play_cursor = sound_buffer.get_play_cursor();
    let Ok(play_cursor) = play_cursor else {
        return running_sample_index;
    };

    let write_offset = running_sample_index * BYTES_PER_SAMPLE % sound_buffer_size;
    let write_length = if write_offset > play_cursor {
        (sound_buffer_size - write_offset) + play_cursor
    } else {
        play_cursor - write_offset
    };

    let buffer_lock_guard = sound_buffer.lock(write_offset, write_length);
    let Ok(buffer_lock_guard) = buffer_lock_guard else {
        return running_sample_index;
    };

    let updated_index = write_square_wave(
        buffer_lock_guard.region1(),
        buffer_lock_guard.region1_size(),
        running_sample_index,
        volume,
    );

    write_square_wave(
        buffer_lock_guard.region2(),
        buffer_lock_guard.region2_size(),
        updated_index,
        volume,
    )
}

fn write_square_wave(
    region: *mut c_void,
    region_size: u32,
    running_sample_index: u32,
    volume: i16,
) -> u32 {
    let mut sample_out = region.cast::<i16>();
    let mut index = running_sample_index;
    let sample_count = region_size / BYTES_PER_SAMPLE;
    for _ in 0..sample_count {
        let sample_value = if (index / SQUARE_WAVE_MID_PERIOD) % 2 == 1 {
            volume
        } else {
            -volume
        };
        unsafe {
            *sample_out = sample_value;
            sample_out = sample_out.add(1);
            *sample_out = sample_value;
            sample_out = sample_out.add(1);
        }
        index = index.wrapping_add(1);
    }
    index
}

// NOTE: We probably don't want to call this as part of the main game loop since it
// can hang the application if the controller is disconnected.
fn poll_controller_state() -> Option<XINPUT_STATE> {
    for controller_index in 0..XUSER_MAX_COUNT {
        let mut controller_state = XINPUT_STATE::default();
        let result = unsafe { XInputGetState(controller_index, &raw mut controller_state) };
        if result == ERROR_SUCCESS.0 {
            return Some(controller_state);
        }
    }
    None
}
