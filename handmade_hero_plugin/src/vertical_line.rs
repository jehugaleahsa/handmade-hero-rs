use std::ops::{Add, Sub};

use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::rectangle::Rectangle;

#[derive(Debug)]
pub struct VerticalLine<T> {
    bottom: T,
    height: T,
    x: T,
}

impl<T> VerticalLine<T> {
    #[inline]
    #[must_use]
    pub fn new(bottom: T, x: T, height: T) -> Self {
        Self { bottom, height, x }
    }
}

impl<T> VerticalLine<T>
where
    T: Add<Output = T> + Copy,
{
    #[inline]
    #[must_use]
    pub fn top(&self) -> T {
        self.bottom + self.height
    }
}

impl<T> VerticalLine<T>
where
    T: Copy,
{
    #[inline]
    #[must_use]
    pub fn bottom(&self) -> T {
        self.bottom
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> T {
        self.height
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> T {
        self.x
    }
}

impl<T> VerticalLine<T>
where
    T: PartialOrd + Sub<Output = T> + Add<Output = T> + Copy,
{
    pub fn bound_to(&self, bounds: &Rectangle<T>) -> Self {
        let bottom = Self::clamp(self.bottom, bounds.bottom(), bounds.top());
        let x = Self::clamp(self.x, bounds.left(), bounds.right());
        let top = Self::clamp(self.top(), bounds.bottom(), bounds.top());
        let height = top - bottom;
        Self { bottom, height, x }
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

impl<T> VerticalLine<T>
where
    T: Add<Output = T> + Copy,
{
    #[inline]
    #[must_use]
    pub fn moved_to(&self, x: T, y: T) -> Self {
        Self {
            bottom: y,
            height: self.height,
            x,
        }
    }

    #[inline]
    #[must_use]
    pub fn shifted(&self, delta_x: T, delta_y: T) -> Self {
        self.moved_to(self.x + delta_x, self.bottom + delta_y)
    }
}

impl VerticalLine<f32> {
    /// # Errors
    /// An error is returned if any of the `f32` positions cannot be converted to a `usize`.
    #[inline]
    pub fn round_to_usize(&self) -> Result<VerticalLine<usize>> {
        let bottom = Self::round_safe(self.bottom)?;
        let height = Self::round_safe(self.height)?;
        let x = Self::round_safe(self.x)?;
        let result = VerticalLine { bottom, height, x };
        Ok(result)
    }

    fn round_safe(value: f32) -> Result<usize> {
        if value < 0f32 {
            return Err(ApplicationError::new("Vertical line with negative value"));
        }
        let rounded = value.round();
        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let truncated = rounded as usize;
        #[expect(clippy::float_cmp)]
        #[expect(clippy::cast_precision_loss)]
        if rounded != truncated as f32 {
            return Err(ApplicationError::new(
                "Vertical line value cannot be converted to usize",
            ));
        }
        Ok(truncated)
    }
}
