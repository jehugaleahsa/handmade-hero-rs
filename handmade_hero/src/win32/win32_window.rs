use std::ffi::c_void;

use handmade_hero_interface::{
    back_buffer::BackBuffer, color::Color, narrow_unsigned, units::si::length::pixel,
};
use uom::si::{
    information::{bit, byte},
    u32::Information,
};
use windows::Win32::Foundation::{COLORREF, FALSE, HINSTANCE, HWND, POINT, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLACKNESS, BeginPaint, ClientToScreen, DIB_RGB_COLORS,
    EndPaint, GetDC, HDC, PAINTSTRUCT, PatBlt, ReleaseDC, SRCCOPY, StretchDIBits,
};
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, GWL_STYLE, GetClientRect,
    GetWindowLongPtrW, GetWindowPlacement, HWND_TOP, IDC_ARROW, LWA_ALPHA, LoadCursorW,
    RegisterClassW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
    SWP_NOZORDER, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPlacement, SetWindowPos,
    WINDOW_EX_STYLE, WINDOW_STYLE, WINDOWPLACEMENT, WNDCLASSW, WNDPROC, WS_EX_LAYERED,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::{Error, HSTRING, PCWSTR, Result as Win32Result, w};

// The frame size depends on these styles, so creating the window and measuring its frame at a
// new DPI must use the same values.
const FRAME_STYLE: WINDOW_STYLE = WS_OVERLAPPEDWINDOW;
const EXTENDED_STYLE: WINDOW_EX_STYLE = WS_EX_LAYERED;

#[derive(Debug)]
pub struct Win32Window {
    bitmap_info: BITMAPINFO,
    window_handle: HWND,
    /// Where the window sat before going fullscreen, so leaving fullscreen can put it back.
    windowed_placement: Option<WINDOWPLACEMENT>,
}

impl Win32Window {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let bitmap_info = Self::initialize_bitmap_info();
        Win32Window {
            bitmap_info,
            window_handle: HWND::default(),
            windowed_placement: None,
        }
    }

    fn initialize_bitmap_info() -> BITMAPINFO {
        // We configure these header field here once since they never change after set.
        let mut bitmap_info = BITMAPINFO::default();
        let header = &mut bitmap_info.bmiHeader;
        header.biSize = narrow_unsigned!(size_of::<BITMAPINFOHEADER>() => u32);
        header.biPlanes = 1;
        let byte_count = narrow_unsigned!(size_of::<Color<u8>>() => u32);
        #[expect(clippy::cast_possible_truncation)]
        let bit_count = Information::new::<byte>(byte_count).get::<bit>() as u16;
        header.biBitCount = bit_count;
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
        title: &str,
        width: u16,
        height: u16,
        application_pointer: *mut c_void,
        window_procedure: WNDPROC,
    ) -> Win32Result<()> {
        let class_name = Self::create_window_class(instance, window_procedure)?;
        self.window_handle = Self::create_win32_window(
            instance,
            class_name,
            title,
            width,
            height,
            application_pointer,
        )?;
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
        title: &str,
        width: u16,
        height: u16,
        application_pointer: *mut c_void,
    ) -> Win32Result<HWND> {
        // Win32 wants a null-terminated UTF-16 string. `HSTRING` owns one, and a reference to it
        // converts to the `PCWSTR` parameter. `CreateWindowExW` copies the title, so the string
        // only has to live for the call.
        let title = HSTRING::from(title);
        let window = unsafe {
            CreateWindowExW(
                EXTENDED_STYLE,
                class_name,
                &title,
                FRAME_STYLE | WS_VISIBLE,
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
        let rectangle = self.current_client_rect()?;
        let header = &mut self.bitmap_info.bmiHeader;
        header.biWidth = Self::rectangle_width(&rectangle);
        header.biHeight = -Self::rectangle_height(&rectangle);
        Ok(())
    }

    /// Computes the window size that keeps the client area at its current size in physical
    /// pixels once the frame is drawn at `dpi`. Only the frame grows or shrinks.
    pub fn window_size_for_dpi(&self, dpi: u32) -> Win32Result<SIZE> {
        let mut rectangle = self.current_client_rect()?;
        unsafe {
            AdjustWindowRectExForDpi(&raw mut rectangle, FRAME_STYLE, false, EXTENDED_STYLE, dpi)?;
        }
        let cx = Self::rectangle_width(&rectangle);
        let cy = Self::rectangle_height(&rectangle);
        let size = SIZE { cx, cy };
        Ok(size)
    }

    pub fn move_to(&self, rectangle: &RECT) -> Win32Result<()> {
        let cx = Self::rectangle_width(rectangle);
        let cy = Self::rectangle_height(rectangle);
        unsafe {
            SetWindowPos(
                self.window_handle,
                None,
                rectangle.left,
                rectangle.top,
                cx,
                cy,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        }
    }

    pub fn enter_fullscreen(&mut self, bounds: &RECT) -> Win32Result<()> {
        if self.windowed_placement.is_none() {
            let length = narrow_unsigned!(size_of::<WINDOWPLACEMENT>() => u32);
            let mut placement = WINDOWPLACEMENT {
                length,
                ..WINDOWPLACEMENT::default()
            };
            unsafe { GetWindowPlacement(self.window_handle, &raw mut placement)? };
            let style = self.style();
            let new_style = style & !FRAME_STYLE;
            self.set_style(new_style);
            self.windowed_placement = Some(placement);
        }
        let cx = Self::rectangle_width(bounds);
        let cy = Self::rectangle_height(bounds);
        unsafe {
            SetWindowPos(
                self.window_handle,
                Some(HWND_TOP),
                bounds.left,
                bounds.top,
                cx,
                cy,
                SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
            )
        }
    }

    pub fn exit_fullscreen(&mut self) -> Win32Result<()> {
        let Some(placement) = self.windowed_placement.take() else {
            return Ok(());
        };
        let style = self.style();
        let new_style = style | FRAME_STYLE;
        self.set_style(new_style);
        unsafe {
            SetWindowPlacement(self.window_handle, &raw const placement)?;
            SetWindowPos(
                self.window_handle,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
            )
        }
    }

    fn style(&self) -> WINDOW_STYLE {
        let style = unsafe { GetWindowLongPtrW(self.window_handle, GWL_STYLE) };
        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        WINDOW_STYLE(style as u32)
    }

    fn set_style(&self, style: WINDOW_STYLE) {
        #[expect(clippy::cast_possible_wrap)]
        let style = style.0 as isize;
        unsafe { SetWindowLongPtrW(self.window_handle, GWL_STYLE, style) };
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
        // The window may have been resized since creation
        let Ok(client_rectangle) = self.current_client_rect() else {
            return;
        };
        let client_width = Self::rectangle_width(&client_rectangle);
        let client_height = Self::rectangle_height(&client_rectangle);
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

    fn current_client_rect(&self) -> Win32Result<RECT> {
        let mut rectangle = RECT::default();
        unsafe { GetClientRect(self.window_handle, &raw mut rectangle)? };
        Ok(rectangle)
    }

    #[inline]
    #[must_use]
    fn rectangle_width(rectangle: &RECT) -> i32 {
        rectangle.right.saturating_sub(rectangle.left)
    }

    #[inline]
    #[must_use]
    fn rectangle_height(rectangle: &RECT) -> i32 {
        rectangle.bottom.saturating_sub(rectangle.top)
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
