mod application;
mod direct_sound;
mod direct_sound_buffer;
mod direct_sound_buffer_lock_guard;
mod performance_counter;
mod pixel;

use crate::application::Application;
use crate::direct_sound::{
    DirectSound, BYTES_PER_SAMPLE, SAMPLES_PER_SECOND, SOUND_BUFFER_SIZE, VOLUME,
};
use crate::direct_sound_buffer::DirectSoundBuffer;
#[cfg(debug_assertions)]
use crate::performance_counter::PerformanceCounter;
use std::cmp::Ordering;
use std::f32::consts::PI;
use std::ffi::c_void;
use windows::core::{w, Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_SUCCESS, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::XboxController::{
    XInputGetState, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE, XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE,
    XINPUT_STATE, XUSER_MAX_COUNT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetWindowLongPtrW, LoadCursorW, PeekMessageW,
    RegisterClassW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    GWL_USERDATA, IDC_ARROW, MSG, PM_REMOVE, WINDOW_EX_STYLE,
    WM_NCCREATE, WM_QUIT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

struct SoundOutputState {
    hertz: u32,
    index: u32,
    theta: f32,
    wave_period: u32,
    latency: u32,
}

fn main() -> Result<()> {
    let instance = get_instance()?;
    let class_name = create_window_class(instance)?;

    let mut application = Application::new();
    let window_handle = create_window(instance, class_name, &mut application)?;

    application.resize_buffer(720, 480).unwrap_or(LRESULT(0));

    let direct_sound = DirectSound::initialize(window_handle).ok();
    let mut sound_buffer = direct_sound
        .as_ref()
        .and_then(|ds| ds.create_buffer(SOUND_BUFFER_SIZE).ok());

    run_application_loop(&mut application, window_handle, &mut sound_buffer, VOLUME)?;

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
    sound_buffer: &mut Option<DirectSoundBuffer<'_>>,
    volume: i16,
) -> Result<()> {
    let mut sound_output = SoundOutputState {
        hertz: 256,
        index: 0,
        theta: 0f32,
        wave_period: SAMPLES_PER_SECOND / 256,
        latency: SAMPLES_PER_SECOND / 15,
    };

    fill_sound_buffer(sound_buffer, &mut sound_output, volume);
    if let Some(sound_buffer) = sound_buffer {
        sound_buffer.play_looping().unwrap_or(()); // Ignore errors
    }

    #[cfg(debug_assertions)]
    let mut counter = PerformanceCounter::start();
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
            #[allow(clippy::cast_possible_wrap)]
            const LEFT_THUMB_DEAD_ZONE: i16 = XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE.0 as i16;
            #[allow(clippy::cast_possible_wrap)]
            const RIGHT_THUMB_DEAD_ZONE: i16 = XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE.0 as i16;
            let gamepad = &controller_state.Gamepad;
            let shift_x = -(gamepad.sThumbLX / LEFT_THUMB_DEAD_ZONE
                + gamepad.sThumbRX / RIGHT_THUMB_DEAD_ZONE);
            application.shift_x(shift_x);
            let shift_y =
                gamepad.sThumbLY / LEFT_THUMB_DEAD_ZONE + gamepad.sThumbRY / RIGHT_THUMB_DEAD_ZONE;
            application.shift_y(shift_y);
            let left_thumb_y_ratio = f32::from(gamepad.sThumbLY) / f32::from(i16::MAX);
            let right_thumb_y_ratio = f32::from(gamepad.sThumbRY) / f32::from(i16::MAX);
            let thumb_y_ratio = left_thumb_y_ratio + right_thumb_y_ratio / 2.0f32;
            #[allow(clippy::cast_sign_loss)]
            #[allow(clippy::cast_possible_truncation)]
            let hertz = (512.0f32 + (256.0f32 * thumb_y_ratio)) as u32;
            sound_output.hertz = hertz;
            let wave_period = SAMPLES_PER_SECOND / hertz;
            sound_output.wave_period = wave_period;
        } else {
            application.shift_x(1);
            application.shift_y(1);
        }

        application.update_display(window_handle)?;

        fill_sound_buffer(sound_buffer, &mut sound_output, volume);

        #[cfg(debug_assertions)]
        display_metrics(&mut counter);
    }
}

fn fill_sound_buffer(
    sound_buffer: &mut Option<DirectSoundBuffer<'_>>,
    sound_output: &mut SoundOutputState,
    volume: i16,
) {
    let Some(sound_buffer) = sound_buffer else {
        return;
    };
    let play_cursor = sound_buffer.get_play_cursor();
    let Ok(play_cursor) = play_cursor else {
        return;
    };

    let write_offset = (sound_output.index * BYTES_PER_SAMPLE) % sound_buffer.length();
    let target_cursor =
        (play_cursor + (sound_output.latency * BYTES_PER_SAMPLE)) % sound_buffer.length();
    let write_length = match write_offset.cmp(&target_cursor) {
        Ordering::Greater => (sound_buffer.length() - write_offset) + target_cursor,
        Ordering::Less => target_cursor - write_offset,
        Ordering::Equal => 0,
    };

    let buffer_lock_guard = sound_buffer.lock(write_offset, write_length);
    let Ok(buffer_lock_guard) = buffer_lock_guard else {
        return;
    };

    write_wave(
        buffer_lock_guard.region1(),
        buffer_lock_guard.region1_size(),
        sound_output,
        volume,
    );

    write_wave(
        buffer_lock_guard.region2(),
        buffer_lock_guard.region2_size(),
        sound_output,
        volume,
    );
}

fn write_wave(
    region: *mut c_void,
    region_size: u32,
    sound_output: &mut SoundOutputState,
    volume: i16,
) {
    let mut sample_out = region.cast::<i16>();
    let sample_count = region_size / BYTES_PER_SAMPLE;
    assert_eq!(sample_count * BYTES_PER_SAMPLE, region_size);

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    let time_delta = 2.0f32 * PI / sound_output.wave_period as f32;

    for _ in 0..sample_count {
        #[allow(clippy::cast_precision_loss)]
        let sine_value = sound_output.theta.sin();
        #[allow(clippy::cast_possible_truncation)]
        let sample_value = (sine_value * f32::from(volume)) as i16;
        unsafe {
            *sample_out = sample_value;
            sample_out = sample_out.add(1);
            *sample_out = sample_value;
            sample_out = sample_out.add(1);
        }
        sound_output.theta += time_delta;
        sound_output.index = sound_output.index.wrapping_add(1);
    }
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

#[cfg(debug_assertions)]
fn display_metrics(counter: &mut PerformanceCounter) {
    let metrics = counter.restart();
    let cycles_elapsed = metrics.elapsed_cycles();
    #[allow(clippy::cast_precision_loss)]
    let megacycles_elapsed = cycles_elapsed as f64 / 1_000_000.0;

    let time_elapsed = metrics.elapsed_time();
    #[allow(clippy::cast_precision_loss)]
    let ms_per_frame = time_elapsed.as_micros() as f64 / 1_000.0;
    #[allow(clippy::cast_precision_loss)]
    let frames_per_second = 1_000.0 / ms_per_frame;

    println!("{ms_per_frame:.2}ms/f, {frames_per_second:.2}f/s, {megacycles_elapsed:.2}Mc/f");
}
