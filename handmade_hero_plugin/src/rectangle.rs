use handmade_hero_interface::application_error::{ApplicationError, Result};
use std::ops::{Add, Sub};

#[derive(Debug)]
pub struct Rectangle<T> {
    top: T,
    left: T,
    bottom: T,
    right: T,
}

#[allow(unused)]
impl<T> Rectangle<T>
where
    T: Add<Output = T> + Sub<Output = T> + PartialOrd + Copy,
{
    #[must_use]
    #[inline]
    pub fn new(top: T, left: T, height: T, width: T) -> Self {
        Self {
            top,
            left,
            bottom: top + height,
            right: left + width,
        }
    }

    #[inline]
    #[must_use]
    pub fn top(&self) -> T {
        self.top
    }

    #[inline]
    #[must_use]
    pub fn left(&self) -> T {
        self.left
    }

    #[inline]
    #[must_use]
    pub fn bottom(&self) -> T {
        self.bottom
    }

    #[inline]
    #[must_use]
    pub fn right(&self) -> T {
        self.right
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> T {
        self.right - self.left
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> T {
        self.bottom - self.top
    }
}

impl<T> Rectangle<T>
where
    T: PartialOrd + Copy,
{
    #[inline]
    #[must_use]
    pub fn bound_to(&self, other: &Self) -> Self {
        Self {
            top: Self::clamp(self.top, other.top, other.bottom),
            left: Self::clamp(self.left, other.left, other.right),
            bottom: Self::clamp(self.bottom, other.top, other.bottom),
            right: Self::clamp(self.right, other.left, other.right),
        }
    }

    fn clamp(value: T, min: T, max: T) -> T {
        // We're assuming we aren't dealing with NaN, Inf, or -Inf.
        let mut result = value;
        if value < min {
            result = min;
        }
        if result > max {
            result = max;
        }
        result
    }
}

impl Rectangle<f32> {
    #[inline]
    pub fn round_to_usize(&self) -> Result<Rectangle<usize>> {
        let top = Self::round_safe(self.top)?;
        let left = Self::round_safe(self.left)?;
        let bottom = Self::round_safe(self.bottom)?;
        let right = Self::round_safe(self.right)?;
        Ok(Rectangle {
            top,
            left,
            bottom,
            right,
        })
    }

    fn round_safe(value: f32) -> Result<usize> {
        if value < 0f32 {
            return Err(ApplicationError::new("Rectangle with negative vertex"));
        }
        let rounded = value.round();
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let truncated = rounded as usize;
        #[allow(clippy::float_cmp)]
        #[allow(clippy::cast_precision_loss)]
        if rounded != truncated as f32 {
            return Err(ApplicationError::new(
                "Rectangle vertex cannot be converted to usize",
            ));
        }
        Ok(truncated)
    }
}
