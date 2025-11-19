use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Color<T> {
    blue: T,
    green: T,
    red: T,
    alpha: T,
}

impl<T> Color<T> {
    #[inline]
    #[must_use]
    pub fn red(self) -> T {
        self.red
    }

    #[inline]
    #[must_use]
    pub fn green(self) -> T {
        self.green
    }

    #[inline]
    #[must_use]
    pub fn blue(self) -> T {
        self.blue
    }

    #[inline]
    #[must_use]
    pub fn alpha(self) -> T {
        self.alpha
    }
}

impl Color<u8> {
    #[must_use]
    #[inline]
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 0,
        }
    }
}

impl Color<f32> {
    #[inline]
    fn convert_to_f32(component: u8) -> f32 {
        f32::from(component) / f32::from(u8::MAX)
    }

    #[inline]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn convert_to_u8(component: f32) -> u8 {
        (component * f32::from(u8::MAX)) as u8
    }
}

impl From<Color<u8>> for u32 {
    #[inline]
    fn from(value: Color<u8>) -> Self {
        (u32::from(value.alpha) << 24)
            | (u32::from(value.red) << 16)
            | (u32::from(value.green) << 8)
            | u32::from(value.blue)
    }
}

impl From<Color<f32>> for Color<u8> {
    #[inline]
    fn from(value: Color<f32>) -> Self {
        Color::from_rgb(
            Color::convert_to_u8(value.red),
            Color::convert_to_u8(value.green),
            Color::convert_to_u8(value.blue),
        )
    }
}

impl From<Color<u8>> for Color<f32> {
    #[inline]
    fn from(value: Color<u8>) -> Self {
        Self {
            red: Self::convert_to_f32(value.red()),
            green: Self::convert_to_f32(value.green()),
            blue: Self::convert_to_f32(value.blue()),
            alpha: Self::convert_to_f32(value.alpha()),
        }
    }
}
