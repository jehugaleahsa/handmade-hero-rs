use std::ffi::c_void;

use handmade_hero_interface::{
    back_buffer::BackBuffer, color::Color, narrow_unsigned, units::si::length::pixel,
};
use windows::{
    Win32::{
        Foundation::{COLORREF, FALSE, HINSTANCE, HWND, POINT, RECT},
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLACKNESS, BeginPaint, ClientToScreen,
            DIB_RGB_COLORS, EndPaint, GetDC, HDC, PAINTSTRUCT, PatBlt, ReleaseDC, SRCCOPY,
            StretchDIBits,
        },
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, GetClientRect, IDC_ARROW,
            LWA_ALPHA, LoadCursorW, RegisterClassW, SetLayeredWindowAttributes, WNDCLASSW, WNDPROC,
            WS_EX_LAYERED, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
    core::{Error, PCWSTR, Result as Win32Result, w},
};

const BITS_PER_BYTE: u16 = 8;

#[derive(Debug)]
pub struct Win32Window {
    bitmap_info: BITMAPINFO,
    window_handle: HWND,
}

impl Win32Window {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let bitmap_info = Self::initialize_bitmap_info();
        Win32Window {
            bitmap_info,
            window_handle: HWND::default(),
        }
    }

    fn initialize_bitmap_info() -> BITMAPINFO {
        // We configure these header field here once since they never change after set.
        let mut bitmap_info = BITMAPINFO::default();
        let header = &mut bitmap_info.bmiHeader;
        header.biSize = narrow_unsigned!(size_of::<BITMAPINFOHEADER>() => u32);
        header.biPlanes = 1;
        header.biBitCount = narrow_unsigned!(size_of::<Color<u8>>() => u16) * BITS_PER_BYTE;
        header.biCompression = BI_RGB.0;
        bitmap_info
    }

    #[inline]
    #[must_use]
    pub fn handle(&self) -> HWND {
        self.window_handle
    }

    #[inline]
    #[must_use]
    pub fn client_width(&self) -> i32 {
        self.bitmap_info.bmiHeader.biWidth
    }

    #[inline]
    #[must_use]
    pub fn client_height(&self) -> i32 {
        -self.bitmap_info.bmiHeader.biHeight
    }

    pub fn create_window(
        &mut self,
        instance: HINSTANCE,
        width: u16,
        height: u16,
        application_pointer: *mut c_void,
        window_procedure: WNDPROC,
    ) -> Win32Result<()> {
        let class_name = Self::create_window_class(instance, window_procedure)?;
        self.window_handle =
            Self::create_win32_window(instance, class_name, width, height, application_pointer)?;
        self.set_client_dimensions()?;
        Ok(())
    }

    fn create_window_class(instance: HINSTANCE, window_procedure: WNDPROC) -> Win32Result<PCWSTR> {
        let class_name = w!("Handmade Hero");
        let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };
        let window_class = WNDCLASSW {
            hCursor: cursor,
            hInstance: instance,
            lpszClassName: class_name,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: window_procedure,
            ..Default::default()
        };

        let register_result = unsafe { RegisterClassW(&raw const window_class) };
        if register_result == 0 {
            return Err(Error::from_thread());
        }
        Ok(class_name)
    }

    fn create_win32_window(
        instance: HINSTANCE,
        class_name: PCWSTR,
        width: u16,
        height: u16,
        application_pointer: *mut c_void,
    ) -> Win32Result<HWND> {
        let window = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED,
                class_name,
                w!("Handmade Hero"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                i32::from(width),
                i32::from(height),
                None,
                None,
                Some(instance),
                Some(application_pointer),
            )?
        };
        Ok(window)
    }

    fn set_client_dimensions(&mut self) -> Win32Result<()> {
        let mut rectangle = RECT::default();
        unsafe { GetClientRect(self.window_handle, &raw mut rectangle)? };
        let header = &mut self.bitmap_info.bmiHeader;
        header.biWidth = rectangle.right.saturating_sub(rectangle.left);
        header.biHeight = -rectangle.bottom.saturating_sub(rectangle.top);
        Ok(())
    }

    pub fn set_transparency(&mut self, is_active: bool) -> Win32Result<()> {
        // We make the window slightly transparent when not active to assist with debugging
        let alpha = if is_active { 0xFF } else { 0x90 };
        unsafe {
            SetLayeredWindowAttributes(self.window_handle, COLORREF::default(), alpha, LWA_ALPHA)?;
        }
        Ok(())
    }

    pub fn repaint(&mut self, back_buffer: &BackBuffer) {
        let mut paint_struct = PAINTSTRUCT::default();
        let device_context = unsafe { BeginPaint(self.window_handle, &raw mut paint_struct) };
        self.write_buffer(back_buffer, device_context);
        let _ = unsafe { EndPaint(self.window_handle, &raw mut paint_struct) };
    }

    pub fn draw(&mut self, back_buffer: &BackBuffer) {
        let device_context = unsafe { GetDC(Some(self.window_handle)) };
        self.write_buffer(back_buffer, device_context);
        unsafe { ReleaseDC(Some(self.window_handle), device_context) };
    }

    fn write_buffer(&mut self, back_buffer: &BackBuffer, device_context: HDC) {
        let client_width = self.client_width();
        let client_height = self.client_height();
        self.render_out_of_bounds(device_context, client_width, client_height);

        let bitmap_data = back_buffer.bitmap();
        #[expect(clippy::cast_possible_truncation)]
        let buffer_width = back_buffer.width().get::<pixel>() as i32;
        #[expect(clippy::cast_possible_truncation)]
        let buffer_height = back_buffer.height().get::<pixel>() as i32;

        unsafe {
            StretchDIBits(
                device_context,
                0,
                0,
                client_width,
                client_height,
                0,
                0,
                buffer_width,
                buffer_height,
                Some(bitmap_data),
                &raw const self.bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    }

    // If the client area exceeds our buffer size due to resizing the window,
    // render a black background. We don't stretch the content.
    fn render_out_of_bounds(&self, device_context: HDC, width: i32, height: i32) {
        let client_height = self.client_height();
        let client_width = self.client_width();
        unsafe {
            let _ = PatBlt(
                device_context,
                width,
                0,
                client_width,
                client_height,
                BLACKNESS,
            );
            let _ = PatBlt(
                device_context,
                0,
                height,
                client_width,
                client_height,
                BLACKNESS,
            );
        }
    }

    pub fn client_coordinate(&self) -> Win32Result<POINT> {
        let mut client_coordinate = POINT::default();
        let result = unsafe { ClientToScreen(self.window_handle, &raw mut client_coordinate) };
        if result == FALSE {
            return Err(Error::from_thread());
        }
        Ok(client_coordinate)
    }
}
