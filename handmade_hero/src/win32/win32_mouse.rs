use handmade_hero_interface::point_2d::Point2d;
use windows::{
    Win32::{
        Foundation::POINT,
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyState, VIRTUAL_KEY, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
            },
            WindowsAndMessaging::GetCursorPos,
        },
    },
    core::Result as Win32Result,
};

#[derive(Debug)]
pub struct Win32Mouse {}

impl Win32Mouse {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Win32Mouse {}
    }

    #[expect(clippy::unused_self)]
    pub fn coordinates(&self) -> Win32Result<Point2d<i32>> {
        let mut cursor_coordinate = POINT::default();
        unsafe {
            GetCursorPos(&raw mut cursor_coordinate)?;
        }
        let point = Point2d::from_x_y(cursor_coordinate.x, cursor_coordinate.y);
        Ok(point)
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn is_left(&self) -> bool {
        Self::is_special_key_down(VK_LBUTTON)
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn is_middle(&self) -> bool {
        Self::is_special_key_down(VK_MBUTTON)
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn is_right(&self) -> bool {
        Self::is_special_key_down(VK_RBUTTON)
    }

    #[must_use]
    fn is_special_key_down(key: VIRTUAL_KEY) -> bool {
        let key_state = unsafe { GetKeyState(i32::from(key.0)) };
        let is_down_mask = 1 << 15; // The key is down if the high-order bit is set.
        (key_state & is_down_mask) != 0
    }
}
