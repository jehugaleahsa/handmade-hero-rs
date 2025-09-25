use crate::application::Application;
use crate::application_error::{ApplicationError, Result};
use crate::button_state::ButtonState;
use crate::direct_sound::DirectSound;
use crate::direct_sound_buffer::DirectSoundBuffer;
use crate::input_state::InputState;
#[cfg(debug_assertions)]
use crate::performance_counter::PerformanceCounter;
use crate::pixel::Pixel;
use crate::stereo_sample::StereoSample;
use core::slice;
use std::cmp::Ordering;
use std::ffi::c_void;
use windows::core::{w, Error, Result as Win32Result, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_SUCCESS, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetDC, ReleaseDC, StretchDIBits, BITMAPINFO, BI_RGB, DIB_RGB_COLORS, HDC,
    PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_A, VK_D, VK_DOWN, VK_E, VK_ESCAPE, VK_F4, VK_LEFT, VK_Q, VK_RIGHT, VK_S, VK_UP,
    VK_W,
};
use windows::Win32::UI::Input::XboxController::{
    XInputGetState, XINPUT_GAMEPAD, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B,
    XINPUT_GAMEPAD_BACK, XINPUT_GAMEPAD_BUTTON_FLAGS, XINPUT_GAMEPAD_DPAD_DOWN,
    XINPUT_GAMEPAD_DPAD_LEFT, XINPUT_GAMEPAD_DPAD_RIGHT, XINPUT_GAMEPAD_DPAD_UP,
    XINPUT_GAMEPAD_LEFT_SHOULDER, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE,
    XINPUT_GAMEPAD_RIGHT_SHOULDER, XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE, XINPUT_GAMEPAD_START,
    XINPUT_GAMEPAD_TRIGGER_THRESHOLD, XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE, XUSER_MAX_COUNT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetWindowLongPtrW, LoadCursorW,
    PeekMessageW, PostQuitMessage, RegisterClassW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW,
    CS_VREDRAW, CW_USEDEFAULT, GWL_USERDATA, IDC_ARROW, MSG, PM_REMOVE,
    WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_NCCREATE, WM_PAINT, WM_QUIT,
    WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

#[derive(Debug)]
pub struct Win32Application {
    application: Application,
    window_handle: HWND,
    bitmap_info: BITMAPINFO,
    bitmap_buffer: Option<Vec<Pixel>>,
    sound_buffer: Option<Vec<StereoSample>>,
    sound_index: u32,
    input_state: InputState,
    closing: bool,
}

impl Win32Application {
    pub fn new(application: Application) -> Win32Application {
        Win32Application {
            application,
            window_handle: HWND::default(),
            bitmap_info: BITMAPINFO::default(),
            bitmap_buffer: None,
            sound_buffer: None,
            sound_index: 0,
            input_state: InputState::default(),
            closing: false,
        }
    }

    pub fn create_window(&mut self, width: u16, height: u16) -> Result<()> {
        let instance = Self::get_instance()
            .map_err(|e| ApplicationError::wrap("Could not retrieve the Windows handle.", e))?;
        let class_name = Self::create_window_class(instance)
            .map_err(|e| ApplicationError::wrap("Failed to create the window class.", e))?;

        let window_handle = Self::create_win32_window(instance, class_name, self)
            .map_err(|e| ApplicationError::wrap("Failed to create the window.", e))?;
        self.window_handle = window_handle;

        self.resize_window(width, height)?;

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

    fn resize_window(&mut self, width: u16, height: u16) -> Result<()> {
        let header = &mut self.bitmap_info.bmiHeader;
        header.biSize = u32::try_from(size_of_val(header)).map_err(|e| {
            ApplicationError::wrap("Failed to determine the bitmap information header size.", e)
        })?;
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

        self.application.resize_bitmap(width, height);

        Ok(())
    }

    fn process_windows_message(
        &mut self,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CLOSE | WM_DESTROY => self.destroy_window(),
            WM_PAINT => self.redraw_window(self.window_handle).unwrap_or(LRESULT(0)), // Ignore error
            WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                self.handle_key_press(w_param, l_param)
            }
            _ => unsafe { DefWindowProcW(self.window_handle, message, w_param, l_param) },
        }
    }

    fn redraw_window(&mut self, window_handle: HWND) -> Win32Result<LRESULT> {
        let mut paint_struct = PAINTSTRUCT::default();
        let device_context = unsafe { BeginPaint(window_handle, &raw mut paint_struct) };
        self.write_buffer(device_context, window_handle)?;
        unsafe {
            #[allow(unused_must_use)]
            EndPaint(window_handle, &raw mut paint_struct);
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

        let keyboard = self.input_state.keyboard_mut();
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
        LRESULT(0)
    }

    fn create_win32_window(
        instance: HINSTANCE,
        class_name: PCWSTR,
        application: &Win32Application,
    ) -> Win32Result<HWND> {
        let application_pointer =
            std::ptr::from_ref::<Win32Application>(application).cast::<c_void>();
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

    fn update_display(&mut self) -> Result<()> {
        if self.closing {
            return Ok(());
        }

        if let Some(ref mut bitmap_buffer) = self.bitmap_buffer {
            self.application.render(bitmap_buffer);
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

        let source_width = i32::from(self.application.bitmap_width());
        let source_height = i32::from(self.application.bitmap_height());

        let client_rectangle = Self::get_client_rectangle(window_handle)?;
        let destination_width = Self::calculate_width(&client_rectangle);
        let destination_height = Self::calculate_height(&client_rectangle);

        unsafe {
            let bitmap_data = bitmap_buffer.as_ptr().cast::<c_void>();
            StretchDIBits(
                device_context,
                client_rectangle.left,
                client_rectangle.top,
                destination_width,
                destination_height,
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

    #[inline]
    #[must_use]
    fn calculate_width(rectangle: &RECT) -> i32 {
        rectangle.right - rectangle.left
    }

    #[inline]
    #[must_use]
    fn calculate_height(rectangle: &RECT) -> i32 {
        rectangle.bottom - rectangle.top
    }

    pub fn run(&mut self) -> Result<()> {
        let direct_sound = DirectSound::initialize(self.window_handle).ok();
        let mut sound_buffer = direct_sound.as_ref().and_then(|ds| {
            ds.create_buffer(
                self.application.sound_channel_count(),
                self.application.sound_samples_per_second(),
                self.application.sound_bits_per_sample(),
                self.application.sound_buffer_size(),
            )
            .ok()
        });

        if let Some(ref mut sound_buffer) = sound_buffer {
            self.fill_sound_buffer(sound_buffer);
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

            self.poll_controller_state();

            self.application.handle_input(&self.input_state);

            self.update_display()?;

            if let Some(ref mut sound_buffer) = sound_buffer {
                self.fill_sound_buffer(sound_buffer);
            }

            #[cfg(debug_assertions)]
            display_metrics(&mut counter);
        }
    }

    fn fill_sound_buffer(&mut self, direct_sound_buffer: &mut DirectSoundBuffer<'_>) {
        let play_cursor = direct_sound_buffer.get_play_cursor();
        let Ok(play_cursor) = play_cursor else {
            return;
        };

        let application = &mut self.application;
        let buffer_length = direct_sound_buffer.length();
        let bytes_per_sample = application.sound_bytes_per_sample();
        let write_offset = (self.sound_index * bytes_per_sample) % buffer_length;
        let latency = application.sound_latency();
        let target_cursor = (play_cursor + (latency * bytes_per_sample)) % buffer_length;
        let write_length = match write_offset.cmp(&target_cursor) {
            Ordering::Greater => (buffer_length - write_offset) + target_cursor,
            Ordering::Less => target_cursor - write_offset,
            Ordering::Equal => 0,
        };
        if write_length == 0 {
            return;
        }

        let sample_count = write_length as usize / size_of::<StereoSample>();
        let sound_buffer = self
            .sound_buffer
            .get_or_insert_with(|| vec![StereoSample::default(); buffer_length as usize]);
        let sound_buffer = &mut sound_buffer[..sample_count];
        application.write_sound(sound_buffer);

        let buffer_lock_guard = direct_sound_buffer.lock(write_offset, write_length);
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
        self.sound_index = self.sound_index.wrapping_add(sample_count);
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

    // NOTE: We probably don't want to call this as part of the main game loop since it
    // can hang the application if the controller is disconnected.
    fn poll_controller_state(&mut self) -> Option<XINPUT_STATE> {
        for controller_index in 0..XUSER_MAX_COUNT {
            let mut controller_state = XINPUT_STATE::default();
            let result = unsafe { XInputGetState(controller_index, &raw mut controller_state) };
            let controller = self
                .input_state
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

#[cfg(debug_assertions)]
fn display_metrics(counter: &mut PerformanceCounter) {
    let metrics = counter.restart();
    let cycles_elapsed = metrics.elapsed_cycles();
    #[allow(clippy::cast_precision_loss)]
    let megacycles_elapsed = cycles_elapsed as f64 / 1_000_000.0;

    let time_elapsed = metrics.elapsed_time();
    #[allow(clippy::cast_precision_loss)]
    let ms_per_frame = time_elapsed.as_micros() as f64 / 1_000.0;
    let frames_per_second = 1_000.0 / ms_per_frame;

    println!("{ms_per_frame:.2}ms/f, {frames_per_second:.2}f/s, {megacycles_elapsed:.2}Mc/f");
}
