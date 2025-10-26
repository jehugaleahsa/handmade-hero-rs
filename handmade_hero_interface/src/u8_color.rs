#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct U8Color {
    blue: u8,
    green: u8,
    red: u8,
    alpha: u8,
}

impl U8Color {
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

    #[inline]
    #[must_use]
    pub fn red(self) -> u8 {
        self.red
    }

    #[inline]
    #[must_use]
    pub fn green(self) -> u8 {
        self.green
    }

    #[inline]
    #[must_use]
    pub fn blue(self) -> u8 {
        self.blue
    }

    #[inline]
    #[must_use]
    pub fn alpha(self) -> u8 {
        self.alpha
    }
}

impl From<U8Color> for u32 {
    #[inline]
    fn from(value: U8Color) -> Self {
        (u32::from(value.alpha) << 24)
            | (u32::from(value.red) << 16)
            | (u32::from(value.green) << 8)
            | u32::from(value.blue)
    }
}
