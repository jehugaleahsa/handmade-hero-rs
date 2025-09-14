use crate::application::Application;
use crate::application_error::{ApplicationError, Result};
use crate::direct_sound::DirectSound;
use crate::direct_sound_buffer::DirectSoundBuffer;
use crate::performance_counter::PerformanceCounter;
use crate::pixel::Pixel;
use std::cmp::Ordering;
use std::f32::consts::PI;
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
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_A, VK_D, VK_F4, VK_S, VK_W};
use windows::Win32::UI::Input::XboxController::{
    XInputGetState, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE, XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE,
    XINPUT_STATE, XUSER_MAX_COUNT,
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
    bitmap_buffer: *mut c_void,
    closing: bool,
}

impl Win32Application {
    pub fn new(application: Application) -> Win32Application {
        Win32Application {
            application,
            window_handle: HWND::default(),
            bitmap_info: BITMAPINFO::default(),
            bitmap_buffer: std::ptr::null_mut(),
            closing: false,
        }
    }

    pub fn create_window(&mut self, width: u32, height: u32) -> Result<()> {
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

    fn resize_window(&mut self, width: u32, height: u32) -> Result<()> {
        let header = &mut self.bitmap_info.bmiHeader;
        header.biSize = u32::try_from(size_of_val(header)).map_err(|e| {
            ApplicationError::wrap("Failed to determine the bitmap information header size.", e)
        })?;
        header.biWidth = i32::try_from(width)
            .map_err(|e| ApplicationError::wrap("Failed to set the bitmap width.", e))?;
        header.biHeight = -i32::try_from(height)
            .map_err(|e| ApplicationError::wrap("Failed to set the bitmap height.", e))?;
        header.biPlanes = 1;
        header.biBitCount = 32;
        header.biCompression = BI_RGB.0;

        let old_buffer = self.bitmap_buffer;
        let memory_size = width as usize * height as usize * size_of::<Pixel>();
        self.bitmap_buffer =
            unsafe { VirtualAlloc(None, memory_size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) };

        self.application.resize_bitmap(width, height);

        if !old_buffer.is_null() && old_buffer != self.bitmap_buffer {
            unsafe {
                VirtualFree(old_buffer, 0, MEM_RELEASE)
                    .map_err(|e| ApplicationError::wrap("Failed to free the window buffer.", e))?;
            }
        }

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
        #[allow(unused_must_use)]
        unsafe {
            EndPaint(window_handle, &raw mut paint_struct)
        };
        Ok(LRESULT(0))
    }

    fn destroy_window(&mut self) -> LRESULT {
        self.closing = true;
        unsafe { PostQuitMessage(0) };
        LRESULT(0)
    }

    fn handle_key_press(&mut self, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        #[allow(clippy::cast_possible_truncation)]
        let virtual_key = VIRTUAL_KEY(w_param.0 as u16);

        // Allow exiting with ALT+F4
        let is_alt_down = (l_param.0 & (1 << 29)) != 0;
        if is_alt_down && virtual_key == VK_F4 {
            return self.destroy_window();
        }

        let was_down = (l_param.0 & (1 << 30)) != 0;
        let is_down = (l_param.0 & (1 << 31)) == 0;
        if was_down == is_down {
            // Ignore repeated messages
            //return LRESULT(0);
        }

        match virtual_key {
            VK_W => self.application.shift_y(-10),
            VK_A => self.application.shift_x(-10),
            VK_S => self.application.shift_y(10),
            VK_D => self.application.shift_x(10),
            _ => {}
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

        self.application.render(self.bitmap_buffer);
        let device_context = unsafe { GetDC(Some(self.window_handle)) };
        self.write_buffer(device_context, self.window_handle)
            .map_err(|e| ApplicationError::wrap("Failed to write the render to the buffer.", e))?;
        unsafe { ReleaseDC(Some(self.window_handle), device_context) };
        Ok(())
    }

    fn write_buffer(&mut self, device_context: HDC, window_handle: HWND) -> Win32Result<()> {
        #[allow(clippy::cast_possible_wrap)]
        let source_width = self.application.bitmap_width() as i32;
        #[allow(clippy::cast_possible_wrap)]
        let source_height = self.application.bitmap_height() as i32;

        let client_rectangle = Self::get_client_rectangle(window_handle)?;
        let destination_width = Self::calculate_width(&client_rectangle);
        let destination_height = Self::calculate_height(&client_rectangle);

        unsafe {
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
                Some(self.bitmap_buffer),
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
    fn calculate_width(rectangle: &RECT) -> i32 {
        rectangle.right - rectangle.left
    }

    #[inline]
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

        self.fill_sound_buffer(&mut sound_buffer);

        if let Some(ref sound_buffer) = sound_buffer {
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

            let application = &mut self.application;
            if let Some(controller_state) = poll_controller_state() {
                #[allow(clippy::cast_possible_wrap)]
                const LEFT_THUMB_DEAD_ZONE: i16 = XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE.0 as i16;
                #[allow(clippy::cast_possible_wrap)]
                const RIGHT_THUMB_DEAD_ZONE: i16 = XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE.0 as i16;
                let gamepad = &controller_state.Gamepad;
                let shift_x = -(gamepad.sThumbLX / LEFT_THUMB_DEAD_ZONE
                    + gamepad.sThumbRX / RIGHT_THUMB_DEAD_ZONE);
                application.shift_x(shift_x);
                let shift_y = gamepad.sThumbLY / LEFT_THUMB_DEAD_ZONE
                    + gamepad.sThumbRY / RIGHT_THUMB_DEAD_ZONE;
                application.shift_y(shift_y);
                let left_thumb_y_ratio = f32::from(gamepad.sThumbLY) / f32::from(i16::MAX);
                let right_thumb_y_ratio = f32::from(gamepad.sThumbRY) / f32::from(i16::MAX);
                let thumb_y_ratio = left_thumb_y_ratio + right_thumb_y_ratio / 2.0f32;
                #[allow(clippy::cast_sign_loss)]
                #[allow(clippy::cast_possible_truncation)]
                let hertz = (512.0f32 + (256.0f32 * thumb_y_ratio)) as u32;
                application.set_sound_hertz(hertz);
            } else {
                application.shift_x(1);
                application.shift_y(1);
            }

            self.update_display()?;

            self.fill_sound_buffer(&mut sound_buffer);

            #[cfg(debug_assertions)]
            display_metrics(&mut counter);
        }
    }

    fn fill_sound_buffer(&mut self, sound_buffer: &mut Option<DirectSoundBuffer<'_>>) {
        let Some(sound_buffer) = sound_buffer else {
            return;
        };
        let play_cursor = sound_buffer.get_play_cursor();
        let Ok(play_cursor) = play_cursor else {
            return;
        };

        let application = &self.application;
        let write_offset = (application.sound_index() * application.sound_bytes_per_sample())
            % sound_buffer.length();
        let target_cursor = (play_cursor
            + (application.sound_latency() * application.sound_bytes_per_sample()))
            % sound_buffer.length();
        let write_length = match write_offset.cmp(&target_cursor) {
            Ordering::Greater => (sound_buffer.length() - write_offset) + target_cursor,
            Ordering::Less => target_cursor - write_offset,
            Ordering::Equal => 0,
        };

        let buffer_lock_guard = sound_buffer.lock(write_offset, write_length);
        let Ok(buffer_lock_guard) = buffer_lock_guard else {
            return;
        };

        self.write_wave(
            buffer_lock_guard.region1(),
            buffer_lock_guard.region1_size(),
        );

        self.write_wave(
            buffer_lock_guard.region2(),
            buffer_lock_guard.region2_size(),
        );
    }

    fn write_wave(&mut self, region: *mut c_void, region_size: u32) {
        let application = &mut self.application;
        let mut sample_out = region.cast::<i16>();
        let sample_count = region_size / application.sound_bytes_per_sample();
        assert_eq!(
            sample_count * application.sound_bytes_per_sample(),
            region_size
        );

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let time_delta = 2.0f32 * PI / application.sound_wave_period() as f32;

        for _ in 0..sample_count {
            #[allow(clippy::cast_precision_loss)]
            let sine_value = application.sound_theta().sin();
            #[allow(clippy::cast_possible_truncation)]
            let sample_value = (sine_value * f32::from(application.sound_volume())) as i16;
            unsafe {
                *sample_out = sample_value;
                sample_out = sample_out.add(1);
                *sample_out = sample_value;
                sample_out = sample_out.add(1);
            }
            application.increase_sound_theta(time_delta);
            application.increase_sound_index(1);
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
