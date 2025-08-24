use std::ffi::c_void;
use std::ptr::null_mut;
use windows::core::{w, Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_SUCCESS, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetDC, ReleaseDC, StretchDIBits, BITMAPINFO, BI_RGB, DIB_RGB_COLORS, HDC,
    PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE,
};
use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE, XUSER_MAX_COUNT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetWindowLongPtrW, LoadCursorW,
    PeekMessageW, PostQuitMessage, RegisterClassW, SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW,
    CS_VREDRAW, CW_USEDEFAULT, GWL_USERDATA, IDC_ARROW, MSG, PM_REMOVE,
    WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_NCCREATE, WM_PAINT, WM_QUIT,
    WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

#[repr(C)]
struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Pixel {
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 0,
        }
    }
}

impl From<Pixel> for u32 {
    fn from(value: Pixel) -> Self {
        (u32::from(value.alpha) << 24)
            | (u32::from(value.red) << 16)
            | (u32::from(value.green) << 8)
            | u32::from(value.blue)
    }
}

struct Application {
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

    fn resize_window(&mut self, window_handle: HWND) -> Result<LRESULT> {
        let mut paint_struct = PAINTSTRUCT::default();
        let device_context = unsafe { BeginPaint(window_handle, &raw mut paint_struct) };
        self.write_buffer(device_context, window_handle)?;
        #[allow(unused_must_use)]
        unsafe {
            EndPaint(window_handle, &raw mut paint_struct)
        };
        Ok(LRESULT(0))
    }

    pub fn update_display(&mut self, window_handle: HWND) -> Result<()> {
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

    fn write_buffer(&mut self, device_context: HDC, window_handle: HWND) -> Result<()> {
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

    pub fn resize_buffer(&mut self, width: u32, height: u32) -> Result<LRESULT> {
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
        self.bitmap_buffer = unsafe { VirtualAlloc(None, memory_size, MEM_COMMIT, PAGE_READWRITE) };

        Ok(LRESULT(0))
    }

    fn get_client_rectangle(window_handle: HWND) -> Result<RECT> {
        let mut client_rectangle = RECT::default();
        unsafe { GetClientRect(window_handle, &raw mut client_rectangle)? };
        Ok(client_rectangle)
    }

    fn calculate_width(rectangle: &RECT) -> i32 {
        rectangle.right - rectangle.left
    }

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
        let was_down = (l_param.0 & (1 << 30)) != 0;
        let is_down = (l_param.0 & (1 << 31)) == 0;
        if was_down == is_down {
            // Ignore repeated messages
            return LRESULT(0);
        }
        #[allow(clippy::cast_possible_truncation)]
        match w_param.0 as u8 {
            b'W' => self.shift_y(-10),
            b'A' => self.shift_x(-10),
            b'S' => self.shift_y(10),
            b'D' => self.shift_x(10),
            _ => {}
        }
        LRESULT(0)
    }
}

fn main() -> Result<()> {
    let instance = get_instance()?;
    let class_name = create_window_class(instance)?;

    let mut application = Application::new();
    let window_handle = create_window(instance, class_name, &mut application)?;

    application.resize_buffer(720, 480).unwrap_or(LRESULT(0));

    run_application_loop(&mut application, window_handle)?;

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

    let atom = unsafe { RegisterClassW(&raw const window_class) };
    if atom == 0 {
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

fn run_application_loop(application: &mut Application, window_handle: HWND) -> Result<()> {
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
        }

        application.update_display(window_handle)?;
    }
}

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
