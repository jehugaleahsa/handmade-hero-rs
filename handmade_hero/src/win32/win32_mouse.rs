use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::mouse_state::MouseState;
use handmade_hero_interface::point_2d::Point2d;
use windows::Win32::Foundation::{POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, WHEEL_DELTA, XBUTTON1, XBUTTON2};
use windows::core::Result as Win32Result;

#[derive(Debug)]
pub struct Win32Mouse {
    state: MouseState,
    vertical_wheel_delta: i32,
    horizontal_wheel_delta: i32,
}

impl Win32Mouse {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Win32Mouse {
            state: MouseState::default(),
            vertical_wheel_delta: 0,
            horizontal_wheel_delta: 0,
        }
    }

    #[inline]
    #[must_use]
    pub fn state(&self) -> &MouseState {
        &self.state
    }

    #[inline]
    #[must_use]
    pub fn state_mut(&mut self) -> &mut MouseState {
        &mut self.state
    }

    #[inline]
    pub fn reset_counts(&mut self) {
        self.state.clear();
    }

    #[inline]
    pub fn release_all(&mut self) {
        self.state.release_all();
    }

    #[inline]
    #[must_use]
    pub fn is_any_down(&self) -> bool {
        self.state.is_any_down()
    }

    pub fn x_button_mut(&mut self, w_param: WPARAM) -> Option<&mut ButtonState> {
        let x_button = (w_param.0 >> 16) & 0xFF_FF;
        if x_button == usize::from(XBUTTON1) {
            Some(self.state.x1_mut())
        } else if x_button == usize::from(XBUTTON2) {
            Some(self.state.x2_mut())
        } else {
            None
        }
    }

    /// Records the cursor position relative to the window's client area.
    pub fn capture_position(&mut self, client_coordinate: POINT) -> Win32Result<()> {
        let mouse_coordinate = Self::coordinates()?;
        let x = mouse_coordinate.x().saturating_sub(client_coordinate.x);
        let y = mouse_coordinate.y().saturating_sub(client_coordinate.y);
        self.state.set_x(x);
        self.state.set_y(y);
        Ok(())
    }

    /// Publishes the whole notches scrolled since the last call, keeping any partial notch for
    /// the next call.
    pub fn capture_wheel(&mut self) {
        let vertical_notches = Self::take_notches(&mut self.vertical_wheel_delta);
        let horizontal_notches = Self::take_notches(&mut self.horizontal_wheel_delta);
        self.state.set_vertical_wheel_delta(vertical_notches);
        self.state.set_horizontal_wheel_delta(horizontal_notches);
    }

    fn coordinates() -> Win32Result<Point2d<i32>> {
        let mut cursor_coordinate = POINT::default();
        unsafe {
            GetCursorPos(&raw mut cursor_coordinate)?;
        }
        let point = Point2d::from_x_y(cursor_coordinate.x, cursor_coordinate.y);
        Ok(point)
    }

    #[inline]
    pub fn process_vertical_scroll(&mut self, w_param: WPARAM) {
        self.vertical_wheel_delta += Self::to_scroll_delta(w_param);
    }

    #[inline]
    pub fn process_horizontal_scroll(&mut self, w_param: WPARAM) {
        self.horizontal_wheel_delta += Self::to_scroll_delta(w_param);
    }

    #[must_use]
    fn take_notches(wheel_delta: &mut i32) -> i32 {
        let wheel_delta_per_notch = WHEEL_DELTA.cast_signed();
        let notches = *wheel_delta / wheel_delta_per_notch;
        *wheel_delta %= wheel_delta_per_notch;
        notches
    }

    #[inline]
    #[must_use]
    fn to_scroll_delta(w_param: WPARAM) -> i32 {
        let high_order = i16::try_from((w_param.0 >> 16) & 0xFF_FF).unwrap_or(0);
        i32::from(high_order)
    }
}
