use crate::u8_pixel::U8Pixel;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct F32Pixel {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl F32Pixel {
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

impl From<F32Pixel> for U8Pixel {
    #[inline]
    fn from(value: F32Pixel) -> Self {
        U8Pixel::from_rgb(
            F32Pixel::convert_to_u8(value.red),
            F32Pixel::convert_to_u8(value.green),
            F32Pixel::convert_to_u8(value.blue),
        )
    }
}

impl From<U8Pixel> for F32Pixel {
    #[inline]
    fn from(value: U8Pixel) -> Self {
        Self {
            red: Self::convert_to_f32(value.red()),
            green: Self::convert_to_f32(value.green()),
            blue: Self::convert_to_f32(value.blue()),
            alpha: Self::convert_to_f32(value.alpha()),
        }
    }
}
