use crate::pixel::Pixel;
use std::ffi::c_void;
use std::ptr::null_mut;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetDC, ReleaseDC, StretchDIBits, BITMAPINFO, BI_RGB, DIB_RGB_COLORS, HDC,
    PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_A, VK_D, VK_F4, VK_S, VK_W};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GetClientRect, PostQuitMessage, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP,
    WM_PAINT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Debug)]
pub struct Application {
    x_offset: u32,
    y_offset: u32,
    bitmap_buffer: *mut c_void,
    bitmap_width: u32,
    bitmap_height: u32,
    bitmap_info: BITMAPINFO,
    closing: bool,
}

impl Application {
    pub fn new() -> Self {
        Self {
            x_offset: 0,
            y_offset: 0,
            bitmap_buffer: null_mut(),
            bitmap_width: 0,
            bitmap_height: 0,
            bitmap_info: BITMAPINFO::default(),
            closing: false,
        }
    }

    pub fn process_windows_message(
        &mut self,
        window_handle: HWND,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CLOSE | WM_DESTROY => self.destroy_window(),
            WM_PAINT => self.resize_window(window_handle).unwrap_or(LRESULT(0)), // Ignore error
            WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                self.handle_key_press(w_param, l_param)
            }
            _ => unsafe { DefWindowProcW(window_handle, message, w_param, l_param) },
        }
    }

    fn destroy_window(&mut self) -> LRESULT {
        self.closing = true;
        unsafe { PostQuitMessage(0) };
        LRESULT(0)
    }

    fn resize_window(&mut self, window_handle: HWND) -> windows::core::Result<LRESULT> {
        let mut paint_struct = PAINTSTRUCT::default();
        let device_context = unsafe { BeginPaint(window_handle, &raw mut paint_struct) };
        self.write_buffer(device_context, window_handle)?;
        #[allow(unused_must_use)]
        unsafe {
            EndPaint(window_handle, &raw mut paint_struct)
        };
        Ok(LRESULT(0))
    }

    pub fn update_display(&mut self, window_handle: HWND) -> windows::core::Result<()> {
        if self.closing {
            return Ok(());
        }

        self.render();
        let device_context = unsafe { GetDC(Some(window_handle)) };
        self.write_buffer(device_context, window_handle)?;
        unsafe { ReleaseDC(Some(window_handle), device_context) };
        Ok(())
    }

    fn render(&self) {
        let width = self.bitmap_width;
        let height = self.bitmap_height;
        let mut pixel = self.bitmap_buffer.cast::<u32>();
        for y in 0..height {
            for x in 0..width {
                let color = Pixel::from_rgb(
                    0,
                    (y.wrapping_add(self.y_offset) & 0xFF) as u8,
                    (x.wrapping_add(self.x_offset) & 0xFF) as u8,
                );
                unsafe { *pixel = u32::from(color) };
                pixel = unsafe { pixel.add(1) };
            }
        }
    }

    fn write_buffer(
        &mut self,
        device_context: HDC,
        window_handle: HWND,
    ) -> windows::core::Result<()> {
        #[allow(clippy::cast_possible_wrap)]
        let source_width = self.bitmap_width as i32;
        #[allow(clippy::cast_possible_wrap)]
        let source_height = self.bitmap_height as i32;

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

    pub fn resize_buffer(&mut self, width: u32, height: u32) -> windows::core::Result<LRESULT> {
        if !self.bitmap_buffer.is_null() {
            unsafe { VirtualFree(self.bitmap_buffer, 0, MEM_RELEASE)? };
        }

        self.bitmap_width = width;
        self.bitmap_height = height;

        let header = &mut self.bitmap_info.bmiHeader;
        header.biSize = u32::try_from(size_of_val(header))?;
        header.biWidth = i32::try_from(width)?;
        header.biHeight = -i32::try_from(height)?;
        header.biPlanes = 1;
        header.biBitCount = 32;
        header.biCompression = BI_RGB.0;

        let memory_size = width as usize * height as usize * size_of::<Pixel>();
        self.bitmap_buffer =
            unsafe { VirtualAlloc(None, memory_size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) };

        Ok(LRESULT(0))
    }

    fn get_client_rectangle(window_handle: HWND) -> windows::core::Result<RECT> {
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

    #[allow(clippy::cast_sign_loss)]
    pub fn shift_x(&mut self, shift: i16) {
        if shift.is_negative() {
            self.x_offset = self.x_offset.wrapping_sub(-shift as u32);
        } else {
            self.x_offset = self.x_offset.wrapping_add(shift as u32);
        }
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn shift_y(&mut self, shift: i16) {
        if shift.is_negative() {
            self.y_offset = self.y_offset.wrapping_sub(-shift as u32);
        } else {
            self.y_offset = self.y_offset.wrapping_add(shift as u32);
        }
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
            VK_W => self.shift_y(-10),
            VK_A => self.shift_x(-10),
            VK_S => self.shift_y(10),
            VK_D => self.shift_x(10),
            _ => {}
        }
        LRESULT(0)
    }
}
