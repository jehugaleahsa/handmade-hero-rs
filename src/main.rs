mod application;
mod direct_sound_buffer_lock;
mod pixel;

use crate::application::Application;
use crate::direct_sound_buffer_lock::DirectSoundBufferLock;
use std::ffi::c_void;
use windows::core::{w, Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_SUCCESS, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::Media::Audio::DirectSound::{
    DirectSoundCreate, IDirectSound, IDirectSoundBuffer, DSBCAPS_PRIMARYBUFFER, DSBPLAY_LOOPING,
    DSBUFFERDESC, DSSCL_PRIORITY,
};
use windows::Win32::Media::Audio::{WAVEFORMATEX, WAVE_FORMAT_PCM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE, XUSER_MAX_COUNT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetWindowLongPtrW, LoadCursorW, PeekMessageW,
    RegisterClassW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    GWL_USERDATA, IDC_ARROW, MSG, PM_REMOVE, WINDOW_EX_STYLE,
    WM_NCCREATE, WM_QUIT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

const NUMBER_OF_CHANNELS: u16 = 2;
const BITS_PER_SAMPLE: u16 = 16;
#[allow(clippy::cast_possible_truncation)]
const BYTES_PER_SAMPLE: u32 = size_of::<u32>() as u32;
const SAMPLES_PER_SECOND: u32 = 48_000u32;
const HERTZ: u32 = 30;
const SQUARE_WAVE_PERIOD: u32 = SAMPLES_PER_SECOND / HERTZ;
const SQUARE_WAVE_MID_PERIOD: u32 = SQUARE_WAVE_PERIOD / 2;

fn main() -> Result<()> {
    let instance = get_instance()?;
    let class_name = create_window_class(instance)?;

    let mut application = Application::new();
    let window_handle = create_window(instance, class_name, &mut application)?;

    application.resize_buffer(720, 480).unwrap_or(LRESULT(0));

    #[allow(clippy::cast_possible_truncation)]
    let sound_buffer_size =
        SAMPLES_PER_SECOND * size_of::<u16>() as u32 * u32::from(NUMBER_OF_CHANNELS);
    let direct_sound = initialize_direct_sound(window_handle).unwrap_or(None);
    let mut sound_buffer = None;
    if let Some(ref direct_sound) = direct_sound {
        let mut format = create_buffer_format();
        sound_buffer =
            create_sound_buffer(direct_sound, sound_buffer_size, &mut format).unwrap_or(None); // Ignore errors - run without sound
        if let Some(ref sound_buffer) = sound_buffer {
            unsafe {
                sound_buffer.Play(0, 0, DSBPLAY_LOOPING).unwrap_or(()); // Ignore errors
            }
        }
    }

    run_application_loop(
        &mut application,
        window_handle,
        sound_buffer.as_ref(),
        sound_buffer_size,
        1000,
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

fn initialize_direct_sound(window_handle: HWND) -> Result<Option<IDirectSound>> {
    let mut direct_sound = None;
    unsafe { DirectSoundCreate(None, &raw mut direct_sound, None)? };
    let Some(direct_sound) = direct_sound else {
        return Ok(None);
    };
    unsafe { direct_sound.SetCooperativeLevel(window_handle, DSSCL_PRIORITY)? };

    Ok(Some(direct_sound))
}

fn create_sound_buffer(
    direct_sound: &IDirectSound,
    buffer_size: u32,
    format: &mut WAVEFORMATEX,
) -> Result<Option<IDirectSoundBuffer>> {
    let primary_buffer_description = create_primary_buffer_description();
    let mut primary_buffer = None;
    unsafe {
        direct_sound.CreateSoundBuffer(
            &raw const primary_buffer_description,
            &raw mut primary_buffer,
            None,
        )?;
    }
    let Some(ref primary_buffer) = primary_buffer else {
        return Ok(None);
    };
    unsafe {
        primary_buffer.SetFormat(format)?;
    }

    let secondary_buffer_description = create_secondary_buffer_description(buffer_size, format);
    let mut secondary_buffer = None;
    unsafe {
        direct_sound.CreateSoundBuffer(
            &raw const secondary_buffer_description,
            &raw mut secondary_buffer,
            None,
        )?;
    }

    Ok(secondary_buffer)
}

fn create_primary_buffer_description() -> DSBUFFERDESC {
    let mut description = DSBUFFERDESC::default();
    #[allow(clippy::cast_possible_truncation)]
    let description_size = size_of::<DSBUFFERDESC>() as u32;
    description.dwSize = description_size;
    description.dwFlags = DSBCAPS_PRIMARYBUFFER;
    // NOTE: The buffer size for the primary buffer should be 0.
    description
}

fn create_buffer_format() -> WAVEFORMATEX {
    const BLOCK_ALIGN: u16 = NUMBER_OF_CHANNELS * BITS_PER_SAMPLE / 8;
    #[allow(clippy::cast_possible_truncation)]
    WAVEFORMATEX {
        wFormatTag: WAVE_FORMAT_PCM as u16,
        nChannels: NUMBER_OF_CHANNELS,
        nSamplesPerSec: SAMPLES_PER_SECOND,
        wBitsPerSample: BITS_PER_SAMPLE,
        nBlockAlign: BLOCK_ALIGN,
        nAvgBytesPerSec: SAMPLES_PER_SECOND * u32::from(BLOCK_ALIGN),
        ..Default::default()
    }
}

fn create_secondary_buffer_description(
    buffer_size: u32,
    format: &mut WAVEFORMATEX,
) -> DSBUFFERDESC {
    let mut description = DSBUFFERDESC::default();
    #[allow(clippy::cast_possible_truncation)]
    let description_size = size_of::<DSBUFFERDESC>() as u32;
    description.dwSize = description_size;
    description.dwBufferBytes = buffer_size;
    description.lpwfxFormat = format;
    description
}

fn run_application_loop(
    application: &mut Application,
    window_handle: HWND,
    sound_buffer: Option<&IDirectSoundBuffer>,
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
    sound_buffer: Option<&IDirectSoundBuffer>,
    sound_buffer_size: u32,
    running_sample_index: u32,
    volume: i16,
) -> u32 {
    let Some(sound_buffer) = sound_buffer else {
        return running_sample_index;
    };
    let mut play_cursor = 0u32;
    let mut write_cursor = 0u32;
    let position_result = unsafe {
        sound_buffer.GetCurrentPosition(Some(&raw mut play_cursor), Some(&raw mut write_cursor))
    };
    let Ok(()) = position_result else {
        return running_sample_index;
    };

    let byte_to_lock = running_sample_index * BYTES_PER_SAMPLE % sound_buffer_size;
    let bytes_to_write = if byte_to_lock > play_cursor {
        (sound_buffer_size - byte_to_lock) + play_cursor
    } else {
        play_cursor - byte_to_lock
    };

    let buffer_lock = DirectSoundBufferLock::new(sound_buffer);
    let buffer_lock_guard = buffer_lock.lock(byte_to_lock, bytes_to_write);
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
