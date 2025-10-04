#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Pixel {
    blue: u8,
    green: u8,
    red: u8,
    alpha: u8,
}

impl Pixel {
    #[must_use]
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
    #[inline]
    fn from(value: Pixel) -> Self {
        (u32::from(value.alpha) << 24)
            | (u32::from(value.red) << 16)
            | (u32::from(value.green) << 8)
            | u32::from(value.blue)
    }
}
