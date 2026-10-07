use std::cell::{Cell, Ref, RefCell, RefMut};

use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::key::Key;
use handmade_hero_interface::key_mapping::KeyMapping;
use handmade_hero_interface::keyboard_state::KeyboardState;
use handmade_hero_interface::mouse_state::MouseState;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetCapture, ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, DefWindowProcW, GWL_USERDATA, GetWindowLongPtrW, PostQuitMessage,
    SetWindowLongPtrW, WM_ACTIVATEAPP, WM_CAPTURECHANGED, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE,
    WM_DPICHANGED, WM_GETDPISCALEDSIZE, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_NCCREATE,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETFOCUS, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_WINDOWPOSCHANGED,
    WM_XBUTTONDOWN, WM_XBUTTONUP,
};

use super::win32_key_event::{self, Win32KeyEvent};
use super::win32_monitor::find_current_monitor;
use super::win32_mouse::Win32Mouse;
use super::win32_window::Win32Window;

/// Everything the window procedure reads or changes.
///
/// Windows calls the window procedure from inside some of our own Win32 calls, such as
/// `SetWindowPos` and `ShowWindow`, while the code that made the call is still running. So the
/// window procedure can't reach the application, which that code holds mutably. `run` owns this
/// state instead and only ever shares it, so the window procedure and the game loop both reach it
/// through `&`, and every change goes through a `Cell` or `RefCell`. Holding a `RefCell` borrow
/// across a call that re-enters the window procedure panics, rather than quietly corrupting
/// memory.
#[derive(Debug)]
pub struct Win32State {
    keyboard: RefCell<KeyboardState>,
    key_mapping: KeyMapping,
    mouse: RefCell<Win32Mouse>,
    current_monitor: Cell<HMONITOR>,
    monitors_changed: Cell<bool>,
    pending_activation: Cell<Option<bool>>,
}

impl Win32State {
    #[must_use]
    pub fn new() -> Self {
        Self {
            keyboard: RefCell::new(KeyboardState::new()),
            key_mapping: KeyMapping::default(),
            mouse: RefCell::new(Win32Mouse::new()),
            current_monitor: Cell::new(HMONITOR::default()),
            monitors_changed: Cell::new(false),
            pending_activation: Cell::new(None),
        }
    }

    /// A new frame starts with every half-transition count at zero, while each button keeps
    /// whether it ended the last frame down.
    pub fn reset_counts(&self) {
        self.keyboard.borrow_mut().reset_counts();
        self.mouse.borrow_mut().reset_counts();
    }

    pub fn keyboard(&self) -> Ref<'_, KeyboardState> {
        self.keyboard.borrow()
    }

    pub fn mouse_mut(&self) -> RefMut<'_, Win32Mouse> {
        self.mouse.borrow_mut()
    }

    /// Whether the window landed on another monitor or the monitors were reconfigured since the
    /// last call.
    pub fn take_monitors_changed(&self) -> bool {
        self.monitors_changed.replace(false)
    }

    /// Whether the game gained (`true`) or lost (`false`) focus since the last call.
    pub fn take_activation(&self) -> Option<bool> {
        self.pending_activation.take()
    }

    fn process_message(
        &self,
        window: HWND,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CLOSE | WM_DESTROY => Self::emit_quitting(),
            WM_ACTIVATEAPP => {
                let is_active = w_param.0 != 0;
                self.pending_activation.set(Some(is_active));
                Win32Window::set_transparency(window, is_active).map_or(LRESULT(0), |()| LRESULT(0))
            }
            WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                self.handle_key_press(w_param, l_param)
            }
            WM_KILLFOCUS => {
                // Any key or button still held will be released into some other window, so we
                // would never hear about it. Let go of everything now rather than leave them
                // stuck down, and stop holding the mouse so other windows get their clicks.
                self.keyboard.borrow_mut().release_all();
                self.mouse.borrow_mut().release_all();
                unsafe {
                    let _ = ReleaseCapture();
                }
                LRESULT(0)
            }
            WM_CAPTURECHANGED => {
                // Another window took the mouse, so button releases will no longer reach us.
                if l_param.0 != window.0 as isize {
                    self.mouse.borrow_mut().release_all();
                }
                LRESULT(0)
            }
            WM_SETFOCUS => {
                self.synchronize_keyboard();
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                self.mouse.borrow_mut().process_vertical_scroll(w_param);
                LRESULT(0)
            }
            WM_MOUSEHWHEEL => {
                self.mouse.borrow_mut().process_horizontal_scroll(w_param);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => self.handle_normal_mouse_button(window, |s| s.left_mut(), true),
            WM_LBUTTONUP => self.handle_normal_mouse_button(window, |s| s.left_mut(), false),
            WM_MBUTTONDOWN => self.handle_normal_mouse_button(window, |s| s.middle_mut(), true),
            WM_MBUTTONUP => self.handle_normal_mouse_button(window, |s| s.middle_mut(), false),
            WM_RBUTTONDOWN => self.handle_normal_mouse_button(window, |s| s.right_mut(), true),
            WM_RBUTTONUP => self.handle_normal_mouse_button(window, |s| s.right_mut(), false),
            WM_XBUTTONDOWN => self.handle_mouse_x_button(window, w_param, true),
            WM_XBUTTONUP => self.handle_mouse_x_button(window, w_param, false),
            WM_GETDPISCALEDSIZE => Self::handle_dpi_scaled_size(window, w_param, l_param),
            WM_DPICHANGED => Self::handle_dpi_changed(window, l_param),
            WM_WINDOWPOSCHANGED => {
                // Every move, resize, or show ends here, including Windows placing the window
                // after creation, so this catches the window landing on another monitor.
                self.detect_monitor_change(window);
                // The default handling is what sends WM_MOVE and WM_SIZE, so it still has to run.
                unsafe { DefWindowProcW(window, message, w_param, l_param) }
            }
            WM_DISPLAYCHANGE => {
                // A monitor was connected or disconnected, or one changed its resolution or
                // refresh rate. Any monitor handle the game loop holds may now be stale.
                self.monitors_changed.set(true);
                LRESULT(0)
            }
            // This includes WM_PAINT. The game loop draws every frame, so painting only has to
            // tell Windows the window is up to date, which the default handling does.
            _ => unsafe { DefWindowProcW(window, message, w_param, l_param) },
        }
    }

    fn detect_monitor_change(&self, window: HWND) {
        let current = find_current_monitor(window);
        if self.current_monitor.replace(current) != current {
            self.monitors_changed.set(true);
        }
    }

    fn handle_dpi_scaled_size(window: HWND, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        // Windows asks for our window size at the new DPI before moving the window there. We
        // keep the game the same number of physical pixels on every monitor, so we ask for the
        // current client area plus a frame drawn at the new DPI.
        #[expect(clippy::cast_possible_truncation)]
        let dpi = (w_param.0 & 0xFFFF) as u32;
        let Ok(size) = Win32Window::window_size_for_dpi(window, dpi) else {
            // Returning FALSE lets Windows scale the window by the DPI ratio instead.
            return LRESULT(0);
        };
        let proposed_size = unsafe { &mut *(l_param.0 as *mut SIZE) };
        *proposed_size = size;
        LRESULT(1)
    }

    fn handle_dpi_changed(window: HWND, l_param: LPARAM) -> LRESULT {
        // The suggested rectangle already has the size we asked for in WM_GETDPISCALEDSIZE, and
        // Windows positions it so the resize can't push the window back onto a monitor with the
        // old DPI. Computing our own position risks bouncing between the two monitors.
        let suggested = unsafe { &*(l_param.0 as *const RECT) };
        let _ = Win32Window::move_to(window, suggested);
        LRESULT(0)
    }

    fn handle_normal_mouse_button(
        &self,
        window: HWND,
        button_getter: impl FnOnce(&mut MouseState) -> &mut ButtonState,
        is_down: bool,
    ) -> LRESULT {
        // The borrow ends before the capture changes, since releasing the capture sends
        // WM_CAPTURECHANGED straight back into the window procedure, which borrows the mouse too.
        let is_any_down = {
            let mut mouse = self.mouse.borrow_mut();
            button_getter(mouse.state_mut()).track_down(is_down);
            mouse.is_any_down()
        };
        Self::handle_capture(window, is_any_down);
        LRESULT(0)
    }

    fn handle_mouse_x_button(&self, window: HWND, w_param: WPARAM, is_down: bool) -> LRESULT {
        let is_any_down = {
            let mut mouse = self.mouse.borrow_mut();
            if let Some(button) = mouse.x_button_mut(w_param) {
                button.track_down(is_down);
                Some(mouse.is_any_down())
            } else {
                None
            }
        };
        if let Some(is_any_down) = is_any_down {
            Self::handle_capture(window, is_any_down);
        }
        LRESULT(1) // Unlike the other buttons, Windows expects TRUE for side buttons.
    }

    fn handle_capture(window: HWND, is_any_down: bool) {
        if is_any_down {
            if unsafe { GetCapture() } != window {
                unsafe { SetCapture(window) };
            }
        } else {
            unsafe {
                let _ = ReleaseCapture();
            }
        }
    }

    /// Reconciles the tracked keyboard with what is physically held, for when focus returns.
    ///
    /// A user who ALT+TABs away and comes back with a key already down would otherwise have to
    /// release and re-press it before the game noticed. Errors are ignored: the fallback is the
    /// stale-but-harmless state we already had.
    fn synchronize_keyboard(&self) {
        if let Ok(key_states) = win32_key_event::physical_key_states() {
            self.keyboard
                .borrow_mut()
                .synchronize(&self.key_mapping, key_states);
        }
    }

    fn emit_quitting() -> LRESULT {
        unsafe { PostQuitMessage(0) };
        LRESULT(0)
    }

    /// Translates one key message into the platform-agnostic keyboard.
    ///
    /// This is deliberately the only place Windows key codes appear. Which button a key drives,
    /// and what happens when several keys drive the same button, is the keyboard's business.
    fn handle_key_press(&self, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        let key_event = Win32KeyEvent::from_params(w_param, l_param);
        if key_event.is_repeat() {
            // A held key autorepeats as a stream of key-down messages. Only real changes matter.
            return LRESULT(0);
        }
        let Some(key) = key_event.key() else {
            return LRESULT(0);
        };
        let is_down = key_event.is_down();
        let is_alt_down = {
            let mut keyboard = self.keyboard.borrow_mut();
            keyboard.track_key(&self.key_mapping, key, is_down);
            keyboard.is_alt_down()
        };

        // Allow exiting with ALT+F4. Handling WM_SYSKEYDOWN ourselves means Windows no longer
        // does this for us.
        if key == Key::F4 && is_down && is_alt_down {
            return Self::emit_quitting();
        }
        LRESULT(0)
    }
}

impl Default for Win32State {
    fn default() -> Self {
        Self::new()
    }
}

pub extern "system" fn window_procedure(
    window: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create_struct = unsafe { &*(l_param.0 as *const CREATESTRUCTW) };
        unsafe { SetWindowLongPtrW(window, GWL_USERDATA, create_struct.lpCreateParams as isize) };
        // The default handler is what records the window title, so it still has to run.
        return unsafe { DefWindowProcW(window, message, w_param, l_param) };
    }

    let state_pointer = unsafe { GetWindowLongPtrW(window, GWL_USERDATA) } as *const Win32State;
    if state_pointer.is_null() {
        // A few messages arrive before WM_NCCREATE, so the default handler takes those.
        return unsafe { DefWindowProcW(window, message, w_param, l_param) };
    }

    // `run` owns the state and destroys the window before the state goes away, so the pointer is
    // valid for as long as the window gets messages.
    let state = unsafe { &*state_pointer };
    state.process_message(window, message, w_param, l_param)
}
