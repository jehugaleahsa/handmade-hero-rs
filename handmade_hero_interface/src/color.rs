use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Color<T> {
    blue: T,
    green: T,
    red: T,
    alpha: T,
}

impl<T> Color<T>
where
    T: Default,
{
    #[must_use]
    #[inline]
    pub fn from_rgb(red: T, green: T, blue: T) -> Self {
        Self::from_rgba(red, green, blue, T::default())
    }
}

impl<T> Color<T> {
    #[must_use]
    #[inline]
    pub fn from_rgba(red: T, green: T, blue: T, alpha: T) -> Self {
        Self {
            blue,
            green,
            red,
            alpha,
        }
    }

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

impl Color<f32> {
    #[inline]
    #[must_use]
    fn convert_to_f32(component: u8) -> f32 {
        f32::from(component) / f32::from(u8::MAX)
    }

    #[inline]
    #[must_use]
    #[expect(clippy::cast_sign_loss)]
    #[expect(clippy::cast_possible_truncation)]
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
        let red = Color::convert_to_u8(value.red);
        let green = Color::convert_to_u8(value.green);
        let blue = Color::convert_to_u8(value.blue);
        let alpha = Color::convert_to_u8(value.alpha);
        Color::from_rgba(red, green, blue, alpha)
    }
}

impl From<Color<u8>> for Color<f32> {
    #[inline]
    fn from(value: Color<u8>) -> Self {
        let red = Self::convert_to_f32(value.red());
        let green = Self::convert_to_f32(value.green());
        let blue = Self::convert_to_f32(value.blue());
        let alpha = Self::convert_to_f32(value.alpha());
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::Color;

    #[test]
    fn test_size_of_color_u8_is_4_bytes() {
        assert_eq!(size_of::<Color<u8>>(), 4);
    }
}
