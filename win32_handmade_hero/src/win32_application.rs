use crate::application_error::{ApplicationError, Result};
use crate::application_loader::ApplicationLoader;
use crate::direct_sound::DirectSound;
use crate::direct_sound_buffer::DirectSoundBuffer;
use crate::performance_counter::PerformanceCounter;
use crate::playback_recorder::PlaybackRecorder;
use core::slice;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::pixel::Pixel;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::stereo_sample::StereoSample;
use std::cmp::Ordering;
use std::ffi::c_void;
use std::ops::Div;
use std::path::PathBuf;
use std::time::Duration;
use windows::Win32::Foundation::{
    COLORREF, ERROR_SUCCESS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, RECT, TRUE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BeginPaint, DEVMODEW, DIB_RGB_COLORS, ENUM_CURRENT_SETTINGS, EndPaint,
    EnumDisplaySettingsW, GetDC, HDC, PAINTSTRUCT, ReleaseDC, SRCCOPY, StretchDIBits,
};
use windows::Win32::Media::{TIMERR_NOERROR, timeBeginPeriod};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_A, VK_CONTROL, VK_D, VK_DOWN, VK_E, VK_ESCAPE, VK_F4, VK_L,
    VK_LEFT, VK_Q, VK_RIGHT, VK_S, VK_UP, VK_W,
};
use windows::Win32::UI::Input::XboxController::{
    XINPUT_GAMEPAD, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B, XINPUT_GAMEPAD_BACK,
    XINPUT_GAMEPAD_BUTTON_FLAGS, XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_LEFT,
    XINPUT_GAMEPAD_DPAD_RIGHT, XINPUT_GAMEPAD_DPAD_UP, XINPUT_GAMEPAD_LEFT_SHOULDER,
    XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE, XINPUT_GAMEPAD_RIGHT_SHOULDER,
    XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE, XINPUT_GAMEPAD_START, XINPUT_GAMEPAD_TRIGGER_THRESHOLD,
    XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE, XInputGetState, XUSER_MAX_COUNT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
    DispatchMessageW, GWL_USERDATA, GetClientRect, GetWindowLongPtrW, IDC_ARROW, LWA_ALPHA,
    LoadCursorW, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage, RegisterClassW,
    SetLayeredWindowAttributes, SetWindowLongPtrW, TranslateMessage, WM_ACTIVATEAPP, WM_CLOSE,
    WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_NCCREATE, WM_PAINT, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
    WNDCLASSW, WS_EX_LAYERED, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::{Error, PCWSTR, Result as Win32Result, w};

const DEFAULT_REFRESH_RATE: u32 = 60;

#[derive(Debug)]
pub enum RecordingState {
    None,
    Recording,
    Playing,
}

#[derive(Debug)]
pub struct Win32Application {
    state: GameState,
    input: InputState,
    window_handle: HWND,
    bitmap_info: BITMAPINFO,
    bitmap_buffer: Option<Vec<Pixel>>,
    sound_buffer: Option<Vec<StereoSample>>,
    sound_index: Option<u32>,
    sound_safety_bytes: u32,
    closing: bool,
    recording_state: RecordingState,
}

impl Win32Application {
    pub fn new() -> Win32Application {
        Win32Application {
            state: GameState::new(),
            input: InputState::new(),
            window_handle: HWND::default(),
            bitmap_info: BITMAPINFO::default(),
            bitmap_buffer: None,
            sound_buffer: None,
            sound_index: None,
            sound_safety_bytes: 0,
            closing: false,
            recording_state: RecordingState::None,
        }
    }

    pub fn create_window(&mut self, width: u16, height: u16) -> Result<()> {
        let instance = Self::get_instance()
            .map_err(|e| ApplicationError::wrap("Could not retrieve the Windows handle", e))?;
        let class_name = Self::create_window_class(instance)
            .map_err(|e| ApplicationError::wrap("Failed to create the window class", e))?;

        self.window_handle = self
            .create_win32_window(instance, class_name, width, height)
            .map_err(|e| ApplicationError::wrap("Failed to create the window", e))?;
        self.set_transparency(true)
            .map_err(|e| ApplicationError::wrap("Failed to display the window", e))?;

        self.resize_window(width, height);

        Ok(())
    }

    fn get_instance() -> Win32Result<HINSTANCE> {
        let instance = unsafe { GetModuleHandleW(None)? };
        Ok(instance.into())
    }

    fn create_window_class(instance: HINSTANCE) -> Win32Result<PCWSTR> {
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

    fn resize_window(&mut self, width: u16, height: u16) {
        let header = &mut self.bitmap_info.bmiHeader;
        #[allow(clippy::cast_possible_truncation)]
        let header_size = size_of_val(header) as u32;
        header.biSize = header_size;
        header.biWidth = i32::from(width);
        header.biHeight = -i32::from(height);
        header.biPlanes = 1;
        header.biBitCount = 32;
        header.biCompression = BI_RGB.0;

        let pixel_count = width as usize * height as usize;
        if let Some(ref mut bitmap_buffer) = self.bitmap_buffer {
            match pixel_count.cmp(&bitmap_buffer.len()) {
                Ordering::Greater => bitmap_buffer.resize(pixel_count, Pixel::default()),
                Ordering::Less => bitmap_buffer.truncate(pixel_count),
                Ordering::Equal => {}
            }
        } else {
            self.bitmap_buffer = Some(vec![Pixel::default(); pixel_count]);
        }

        self.state.set_width(width);
        self.state.set_height(height);
    }

    fn process_windows_message(
        &mut self,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CLOSE | WM_DESTROY => self.destroy_window(),
            WM_ACTIVATEAPP => self
                .set_transparency(w_param.0 == TRUE.0 as usize)
                .unwrap_or(LRESULT(0)),
            WM_PAINT => self.redraw_window().unwrap_or(LRESULT(0)), // Ignore error
            WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                self.handle_key_press(w_param, l_param)
            }
            _ => unsafe { DefWindowProcW(self.window_handle, message, w_param, l_param) },
        }
    }

    fn set_transparency(&self, is_active: bool) -> Win32Result<LRESULT> {
        let alpha = if is_active { 0xFF } else { 0x40 };
        unsafe {
            SetLayeredWindowAttributes(self.window_handle, COLORREF::default(), alpha, LWA_ALPHA)?;
        }
        Ok(LRESULT(0))
    }

    fn redraw_window(&mut self) -> Win32Result<LRESULT> {
        let mut paint_struct = PAINTSTRUCT::default();
        let device_context = unsafe { BeginPaint(self.window_handle, &raw mut paint_struct) };
        self.write_buffer(device_context, self.window_handle)?;
        unsafe {
            #[allow(unused_must_use)]
            EndPaint(self.window_handle, &raw mut paint_struct);
        }
        Ok(LRESULT(0))
    }

    fn destroy_window(&mut self) -> LRESULT {
        self.closing = true;
        unsafe { PostQuitMessage(0) };
        LRESULT(0)
    }

    fn handle_key_press(&mut self, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        let was_down = (l_param.0 & (1 << 30)) != 0;
        let is_down = (l_param.0 & (1 << 31)) == 0;
        if was_down == is_down {
            // Ignore repeated messages
            return LRESULT(0);
        }

        #[allow(clippy::cast_possible_truncation)]
        let virtual_key = VIRTUAL_KEY(w_param.0 as u16);

        // Allow exiting with ALT+F4
        let is_alt_down = (l_param.0 & (1 << 29)) != 0;
        if is_alt_down && virtual_key == VK_F4 {
            return self.destroy_window();
        }

        let keyboard = self.input.keyboard_mut();
        let mapped_button = match virtual_key {
            VK_W | VK_UP => Some(keyboard.up_mut()),
            VK_A | VK_LEFT => Some(keyboard.left_mut()),
            VK_S | VK_DOWN => Some(keyboard.down_mut()),
            VK_D | VK_RIGHT => Some(keyboard.right_mut()),
            VK_Q => Some(keyboard.left_shoulder_mut()),
            VK_E => Some(keyboard.right_shoulder_mut()),
            VK_ESCAPE => Some(keyboard.start_mut()),
            _ => None,
        };
        if let Some(mapped_button) = mapped_button {
            mapped_button.set_ended_down(is_down);
            if is_down {
                mapped_button.increment_half_transition_count();
            } else {
                mapped_button.reset_half_transition_count();
            }
        }
        if virtual_key == VK_L && is_down {
            // Hitting 'L' begins a recording sessions.
            // Hitting 'L' again causes the recording session to end.
            // The recording will play back in an infinite loop until CTRL+L is hit.
            let control_state = unsafe { GetKeyState(i32::from(VK_CONTROL.0)) };
            let is_control_down = (control_state & (1 << 15)) != 0;
            match (&self.recording_state, is_control_down) {
                (RecordingState::None | RecordingState::Playing, false) => {
                    self.recording_state = RecordingState::Recording;
                }
                (RecordingState::Recording, false) => {
                    self.recording_state = RecordingState::Playing;
                }
                (_, true) => {
                    self.recording_state = RecordingState::None;
                    keyboard.clear();
                }
            }
        }
        LRESULT(0)
    }

    fn create_win32_window(
        &self,
        instance: HINSTANCE,
        class_name: PCWSTR,
        width: u16,
        height: u16,
    ) -> Win32Result<HWND> {
        let application_pointer = std::ptr::from_ref::<Win32Application>(self).cast::<c_void>();
        unsafe {
            CreateWindowExW(
                WS_EX_LAYERED,
                class_name,
                w!("Handmade Hero"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                i32::from(width.cast_signed()),
                i32::from(height.cast_signed()),
                None,
                None,
                Some(instance),
                Some(application_pointer),
            )
        }
    }

    fn update_display(&mut self) -> Result<()> {
        if self.closing {
            return Ok(());
        }

        let device_context = unsafe { GetDC(Some(self.window_handle)) };
        self.write_buffer(device_context, self.window_handle)
            .map_err(|e| ApplicationError::wrap("Failed to write the render to the buffer.", e))?;
        unsafe { ReleaseDC(Some(self.window_handle), device_context) };

        Ok(())
    }

    fn write_buffer(&mut self, device_context: HDC, window_handle: HWND) -> Win32Result<()> {
        let Some(ref bitmap_buffer) = self.bitmap_buffer else {
            return Ok(());
        };

        let source_width = i32::from(self.state.width());
        let source_height = i32::from(self.state.height());

        let client_rectangle = Self::get_client_rectangle(window_handle)?;

        unsafe {
            let bitmap_data = bitmap_buffer.as_ptr().cast::<c_void>();
            StretchDIBits(
                device_context,
                client_rectangle.left,
                client_rectangle.top,
                source_width,
                source_height,
                0,
                0,
                source_width,
                source_height,
                Some(bitmap_data),
                &raw const self.bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
        Ok(())
    }

    fn get_client_rectangle(window_handle: HWND) -> Win32Result<RECT> {
        let mut client_rectangle = RECT::default();
        unsafe { GetClientRect(window_handle, &raw mut client_rectangle)? };
        Ok(client_rectangle)
    }

    pub fn run(&mut self) -> Result<()> {
        let monitor_refresh_hertz = Self::find_monitor_refresh_hertz();
        let game_update_hertz = monitor_refresh_hertz / 2;

        #[allow(clippy::cast_precision_loss)]
        let target_frame_duration = Duration::from_secs_f32(1.0f32 / game_update_hertz as f32);
        let is_sleep_granular = unsafe {
            // Set the Windows scheduler granularity to 1ms!
            timeBeginPeriod(1) == TIMERR_NOERROR
        };

        let direct_sound = DirectSound::initialize(self.window_handle).ok();
        let mut sound_buffer = direct_sound.as_ref().and_then(|ds| {
            ds.create_buffer(
                self.state.sound_channel_count(),
                self.state.sound_samples_per_second(),
                self.state.sound_bits_per_sample(),
                self.state.sound_buffer_size(),
            )
            .ok()
        });

        if let Some(ref mut sound_buffer) = sound_buffer {
            sound_buffer.play_looping().unwrap_or(()); // Ignore errors
            self.sound_safety_bytes = self.calculate_sound_safety_bytes(game_update_hertz);
        }

        // Center player initially
        let x_center = self
            .state
            .width()
            .div(2)
            .saturating_add(GameState::PLAYER_WIDTH.div(2));
        let y_center = self
            .state
            .height()
            .div(2)
            .saturating_add(GameState::PLAYER_HEIGHT.div(2));
        self.state.set_player_x(x_center);
        self.state.set_player_y(y_center);

        let exe_directory = Self::exe_directory()?;
        let mut loader = ApplicationLoader::new(&exe_directory);
        let mut recorder = PlaybackRecorder::new(&exe_directory);
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

            let application = loader.load()?;
            self.poll_controller_state();

            if let RecordingState::Recording = self.recording_state {
                recorder
                    .record(&self.input, &self.state)
                    .unwrap_or_default(); // Ignore errors
            } else if let RecordingState::Playing = self.recording_state {
                if let Some(state) = recorder.playback().unwrap_or_default() {
                    (self.input, self.state) = state;
                } else {
                    recorder.reset_playback().unwrap_or_default(); // We miss a frame here
                }
            }
            application.process_input(&self.input, &mut self.state);

            if let Some(ref mut bitmap_buffer) = self.bitmap_buffer {
                let mut context = RenderContext::new(&mut self.state, bitmap_buffer);
                application.render(&mut context);
            }

            if let Some(sound_index) = self.sound_index
                && let Some(ref mut sound_buffer) = sound_buffer
            {
                self.fill_sound_buffer(
                    application,
                    sound_buffer,
                    sound_index,
                    game_update_hertz,
                    target_frame_duration,
                    &counter,
                );
            }

            wait_for_framerate(&mut counter, target_frame_duration, is_sleep_granular);

            self.update_display()?;

            // After a single frame, we have a better idea how far away the sound
            // play cursor is from the write cursor. We initialize the sound index
            // as a flag for sound to start being written now that the metrics are
            // recorded.
            if self.sound_index.is_none()
                && let Some(ref sound_buffer) = sound_buffer
            {
                self.sound_index = self.get_sample_index(sound_buffer);
            }
        }
    }

    fn calculate_sound_safety_bytes(&mut self, game_update_hertz: u32) -> u32 {
        let sound_samples_per_second = self.state.sound_samples_per_second();
        let sound_bytes_per_sample = self.state.sound_bytes_per_sample();
        let sound_bytes_per_second = sound_samples_per_second * sound_bytes_per_sample;
        let sound_bytes_per_game_hertz = sound_bytes_per_second / game_update_hertz;
        sound_bytes_per_game_hertz / 2
    }

    fn exe_directory() -> Result<PathBuf> {
        let current_exe_path = std::env::current_exe().map_err(|e| {
            ApplicationError::wrap("Failed to retrieve the current executable path", e)
        })?;
        let current_directory = current_exe_path.parent().ok_or_else(|| {
            ApplicationError::new("Failed to retrieve the current executable parent directory")
        })?;
        Ok(current_directory.to_path_buf())
    }

    fn find_monitor_refresh_hertz() -> u32 {
        #[allow(clippy::cast_possible_truncation)]
        let size = size_of::<DEVMODEW>() as u16;
        let mut mode = DEVMODEW {
            dmSize: size,
            ..DEVMODEW::default()
        };
        let success = unsafe {
            #[allow(unused)]
            EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &raw mut mode)
        };
        if !success.as_bool() {
            return DEFAULT_REFRESH_RATE;
        }
        let frequency = mode.dmDisplayFrequency;
        if frequency == 0 || frequency == 1 {
            return DEFAULT_REFRESH_RATE;
        }
        frequency
    }

    fn fill_sound_buffer(
        &mut self,
        application: &dyn Application,
        direct_sound_buffer: &mut DirectSoundBuffer<'_>,
        sound_index: u32,
        game_update_hertz: u32,
        target_frame_duration: Duration,
        performance_counter: &PerformanceCounter,
    ) {
        let Ok((play_cursor, write_cursor)) = direct_sound_buffer.get_cursors() else {
            return;
        };
        let buffer_length = direct_sound_buffer.length();
        let bytes_per_sample = self.state.sound_bytes_per_sample();
        let write_offset = (sound_index * bytes_per_sample) % buffer_length;

        let safe_write_cursor = write_cursor
            + self.sound_safety_bytes
            + if write_cursor < play_cursor {
                direct_sound_buffer.length()
            } else {
                0
            };
        let samples_per_second = self.state.sound_samples_per_second();
        let frame_time_elapsed = performance_counter.metrics().elapsed_time();
        let remaining_frame_time = if target_frame_duration >= frame_time_elapsed {
            target_frame_duration - frame_time_elapsed
        } else {
            Duration::default()
        };
        let remaining_time_ratio = remaining_frame_time.div_duration_f32(target_frame_duration);
        let bytes_per_frame = (samples_per_second * bytes_per_sample) / game_update_hertz;
        #[allow(clippy::cast_precision_loss)]
        let remaining_bytes = remaining_time_ratio * bytes_per_frame as f32;
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let remaining_bytes = remaining_bytes as u32;
        let expected_frame_boundary_bytes = play_cursor + remaining_bytes;
        let audio_is_latent = safe_write_cursor >= expected_frame_boundary_bytes;
        let target_cursor = if audio_is_latent {
            write_cursor + self.sound_safety_bytes + bytes_per_frame
        } else {
            expected_frame_boundary_bytes + bytes_per_frame
        };
        let target_cursor = target_cursor % buffer_length;
        let bytes_to_write = match write_offset.cmp(&target_cursor) {
            Ordering::Greater => (buffer_length - write_offset) + target_cursor,
            Ordering::Less => target_cursor - write_offset,
            Ordering::Equal => 0,
        };
        if bytes_to_write == 0 {
            return;
        }

        let sample_count = bytes_to_write as usize / size_of::<StereoSample>();
        let sound_buffer = self
            .sound_buffer
            .get_or_insert_with(|| vec![StereoSample::default(); buffer_length as usize]);
        let sound_buffer = &mut sound_buffer[..sample_count];
        let mut context = AudioContext::new(&mut self.state, sound_buffer);
        application.write_sound(&mut context);

        let buffer_lock_guard = direct_sound_buffer.lock(write_offset, bytes_to_write);
        let Ok(buffer_lock_guard) = buffer_lock_guard else {
            return;
        };

        Self::copy_sound_buffer(
            buffer_lock_guard.region1(),
            buffer_lock_guard.region1_size(),
            sound_buffer,
            0,
        );

        Self::copy_sound_buffer(
            buffer_lock_guard.region2(),
            buffer_lock_guard.region2_size(),
            sound_buffer,
            buffer_lock_guard.region1_size(),
        );
        let sample_count = u32::try_from(sample_count)
            .expect("The sound index could not be converted to an unsigned 32-bit integer.");
        self.sound_index = Some(sound_index.wrapping_add(sample_count));
    }

    fn copy_sound_buffer(
        destination: *mut c_void,
        destination_length_in_bytes: u32,
        source: &[StereoSample],
        source_offset_in_bytes: u32,
    ) {
        if destination.is_null() {
            return;
        }
        let sample_count = destination_length_in_bytes as usize / size_of::<StereoSample>();
        let sample_out =
            unsafe { slice::from_raw_parts_mut(destination.cast::<StereoSample>(), sample_count) };
        let source_offset = source_offset_in_bytes as usize / size_of::<StereoSample>();
        let source_end = source_offset + sample_count;
        let source_slice = &source[source_offset..source_end];
        assert_eq!(source_slice.len(), sample_out.len());
        sample_out.copy_from_slice(source_slice);
    }

    fn get_sample_index(&self, direct_sound_buffer: &DirectSoundBuffer<'_>) -> Option<u32> {
        let (_, write_cursor) = direct_sound_buffer.get_cursors().ok()?;
        let bytes_per_sample = self.state.sound_bytes_per_sample();
        let sample_index = write_cursor / bytes_per_sample;
        Some(sample_index)
    }

    // NOTE: We probably don't want to call this as part of the main game loop since it
    // can hang the application if the controller is disconnected.
    fn poll_controller_state(&mut self) -> Option<XINPUT_STATE> {
        for controller_index in 0..XUSER_MAX_COUNT {
            let mut controller_state = XINPUT_STATE::default();
            let result = unsafe { XInputGetState(controller_index, &raw mut controller_state) };
            let controller = self
                .input
                .get_or_insert_controller_mut(controller_index as usize);
            if result == ERROR_SUCCESS.0 {
                let gamepad = &controller_state.Gamepad;
                controller.set_enabled(true);
                Self::set_button_state(controller.a_mut(), gamepad, XINPUT_GAMEPAD_A);
                Self::set_button_state(controller.b_mut(), gamepad, XINPUT_GAMEPAD_B);
                Self::set_button_state(controller.x_mut(), gamepad, XINPUT_GAMEPAD_X);
                Self::set_button_state(controller.y_mut(), gamepad, XINPUT_GAMEPAD_Y);
                Self::set_button_state(controller.start_mut(), gamepad, XINPUT_GAMEPAD_START);
                Self::set_button_state(controller.back_mut(), gamepad, XINPUT_GAMEPAD_BACK);
                Self::set_button_state(controller.up_mut(), gamepad, XINPUT_GAMEPAD_DPAD_UP);
                Self::set_button_state(controller.down_mut(), gamepad, XINPUT_GAMEPAD_DPAD_DOWN);
                Self::set_button_state(controller.left_mut(), gamepad, XINPUT_GAMEPAD_DPAD_LEFT);
                Self::set_button_state(controller.right_mut(), gamepad, XINPUT_GAMEPAD_DPAD_RIGHT);
                Self::set_button_state(
                    controller.left_shoulder_mut(),
                    gamepad,
                    XINPUT_GAMEPAD_LEFT_SHOULDER,
                );
                Self::set_button_state(
                    controller.right_shoulder_mut(),
                    gamepad,
                    XINPUT_GAMEPAD_RIGHT_SHOULDER,
                );

                let left_joystick = controller.left_joystick_mut();
                left_joystick.set_x(Self::thumb_stick_ratio(
                    gamepad.sThumbLX,
                    XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE.0,
                ));
                left_joystick.set_y(Self::thumb_stick_ratio(
                    gamepad.sThumbLY,
                    XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE.0,
                ));
                let right_joystick = controller.right_joystick_mut();
                right_joystick.set_x(Self::thumb_stick_ratio(
                    gamepad.sThumbRX,
                    XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE.0,
                ));
                right_joystick.set_y(Self::thumb_stick_ratio(
                    gamepad.sThumbRY,
                    XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE.0,
                ));

                controller.set_left_trigger_ratio(Self::trigger_ratio(gamepad.bLeftTrigger));
                controller.set_right_trigger_ratio(Self::trigger_ratio(gamepad.bRightTrigger));
            } else {
                controller.set_enabled(false);
            }
        }
        None
    }

    fn set_button_state(
        button_state: &mut ButtonState,
        gamepad: &XINPUT_GAMEPAD,
        button_flag: XINPUT_GAMEPAD_BUTTON_FLAGS,
    ) {
        let is_pressed = Self::is_pressed(gamepad, button_flag);
        let was_pressed = button_state.ended_down();
        if was_pressed == is_pressed {
            button_state.increment_half_transition_count();
        }
        button_state.set_ended_down(is_pressed);
    }

    #[inline]
    #[must_use]
    fn is_pressed(gamepad: &XINPUT_GAMEPAD, button: XINPUT_GAMEPAD_BUTTON_FLAGS) -> bool {
        (gamepad.wButtons & button).0 != 0
    }

    #[inline]
    #[must_use]
    fn thumb_stick_ratio(amount: i16, dead_zone: u16) -> f32 {
        if amount.unsigned_abs() <= dead_zone {
            0f32
        } else if amount < 0 {
            let dead_zone = f32::from(dead_zone);
            -((f32::from(amount) + dead_zone) / (f32::from(i16::MIN) + dead_zone))
        } else {
            let dead_zone = f32::from(dead_zone);
            (f32::from(amount) - dead_zone) / (f32::from(i16::MAX) - dead_zone)
        }
    }

    #[inline]
    #[must_use]
    fn trigger_ratio(amount: u8) -> f32 {
        if u16::from(amount) <= XINPUT_GAMEPAD_TRIGGER_THRESHOLD.0 {
            0f32
        } else {
            let threshold = f32::from(XINPUT_GAMEPAD_TRIGGER_THRESHOLD.0);
            (f32::from(amount) - threshold) / (f32::from(u8::MAX) - threshold)
        }
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
        let application = create_struct.lpCreateParams.cast::<Win32Application>();
        unsafe { SetWindowLongPtrW(window_handle, GWL_USERDATA, application as isize) };
        return LRESULT(1); // Indicate we should proceed with creating the window.
    }

    let application_pointer =
        unsafe { GetWindowLongPtrW(window_handle, GWL_USERDATA) } as *mut Win32Application;
    if application_pointer.is_null() {
        // We're not initialized yet, so just let the default handler run.
        return unsafe { DefWindowProcW(window_handle, message, w_param, l_param) };
    }

    // We keep an Application object alive for the duration of the application.
    // This allows us to maintain state about the application without relying on
    // global variables.
    let application = unsafe { &mut *application_pointer };
    application.process_windows_message(message, w_param, l_param)
}

fn wait_for_framerate(
    counter: &mut PerformanceCounter,
    target_duration: Duration,
    is_sleep_granular: bool,
) {
    let mut metrics = counter.metrics();
    let mut time_elapsed = metrics.elapsed_time();
    while time_elapsed < target_duration {
        if is_sleep_granular {
            let remaining = target_duration - time_elapsed;
            #[allow(clippy::cast_possible_truncation)]
            let remaining = Duration::from_millis(remaining.as_millis() as u64);
            std::thread::sleep(remaining);
        }

        metrics = counter.metrics();
        time_elapsed = metrics.elapsed_time();
    }

    counter.restart();

    #[allow(clippy::cast_precision_loss)]
    let ms_per_frame = time_elapsed.as_micros() as f32 / 1_000.0f32;
    let frames_per_second = 1_000.0f32 / ms_per_frame;

    let cycles_elapsed = metrics.elapsed_cycles();
    #[allow(clippy::cast_precision_loss)]
    let megacycles_elapsed = cycles_elapsed as f32 / 1_000_000.0f32;

    println!("{ms_per_frame:.2}ms/f, {frames_per_second:.2}f/s, {megacycles_elapsed:.2}Mc/f");
}
