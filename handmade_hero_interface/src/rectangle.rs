use crate::application_error::{ApplicationError, Result};
use crate::point_2d::Point2d;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
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
    pub fn new(bottom: T, left: T, height: T, width: T) -> Self {
        Self {
            top: bottom + height,
            left,
            bottom,
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
        self.top - self.bottom
    }

    #[inline]
    #[must_use]
    pub fn moved_to(&self, x: T, y: T) -> Self {
        let right = x + self.width();
        let top = y + self.height();
        Self {
            top,
            left: x,
            right,
            bottom: y,
        }
    }

    #[inline]
    #[must_use]
    pub fn moved_to_point(&self, point: Point2d<T>) -> Self {
        self.moved_to(point.x(), point.y())
    }

    #[inline]
    #[must_use]
    pub fn shifted(&self, delta_x: T, delta_y: T) -> Self {
        self.moved_to(self.left + delta_x, self.bottom + delta_y)
    }

    #[inline]
    #[must_use]
    pub fn resized(&self, height: T, width: T) -> Self {
        Self {
            top: self.bottom + height,
            left: self.left,
            bottom: self.bottom,
            right: self.left + width,
        }
    }

    #[must_use]
    #[inline]
    pub fn contains_point(&self, point: Point2d<T>) -> bool {
        point.x() >= self.left
            && point.x() < self.right
            && point.y() < self.top
            && point.y() >= self.bottom
    }

    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        other.left < self.right
            && other.right > self.left
            && other.bottom < self.top
            && other.top > self.bottom
    }

    #[must_use]
    #[inline]
    pub fn top_left(&self) -> Point2d<T> {
        Point2d::from_x_y(self.left, self.top)
    }

    #[must_use]
    #[inline]
    pub fn bottom_left(&self) -> Point2d<T> {
        Point2d::from_x_y(self.left, self.bottom)
    }

    #[must_use]
    #[inline]
    pub fn top_right(&self) -> Point2d<T> {
        Point2d::from_x_y(self.right, self.top)
    }

    #[must_use]
    #[inline]
    pub fn bottom_right(&self) -> Point2d<T> {
        Point2d::from_x_y(self.right, self.bottom)
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
            top: Self::clamp(self.top, other.bottom, other.top),
            left: Self::clamp(self.left, other.left, other.right),
            bottom: Self::clamp(self.bottom, other.bottom, other.top),
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
    /// # Errors
    /// An error is returned if any of the `f32` positions cannot be converted to a `usize`.
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

#[cfg(test)]
mod tests {
    use crate::rectangle::Rectangle;

    #[test]
    fn test_overlaps_top_left_corner() {
        let first = Rectangle::new(5, 5, 10, 10);
        let second = Rectangle::new(0, 0, 10, 10);
        let overlaps = first.overlaps(&second);
        assert!(overlaps);
        let overlaps = second.overlaps(&first);
        assert!(overlaps);
    }

    #[test]
    fn test_overlaps_bottom_left_corner() {
        let first = Rectangle::new(5, 5, 10, 10);
        let second = Rectangle::new(10, 0, 10, 10);
        let overlaps = first.overlaps(&second);
        assert!(overlaps);
        let overlaps = second.overlaps(&first);
        assert!(overlaps);
    }

    #[test]
    fn test_overlaps_top_right_corner() {
        let first = Rectangle::new(5, 5, 10, 10);
        let second = Rectangle::new(0, 10, 10, 10);
        let overlaps = first.overlaps(&second);
        assert!(overlaps);
        let overlaps = second.overlaps(&first);
        assert!(overlaps);
    }

    #[test]
    fn test_overlaps_bottom_right_corner() {
        let first = Rectangle::new(5, 5, 10, 10);
        let second = Rectangle::new(10, 10, 10, 10);
        let overlaps = first.overlaps(&second);
        assert!(overlaps);
        let overlaps = second.overlaps(&first);
        assert!(overlaps);
    }

    #[test]
    fn test_overlaps_inside() {
        let first = Rectangle::new(0, 0, 20, 20);
        let second = Rectangle::new(5, 5, 5, 5);
        let overlaps = first.overlaps(&second);
        assert!(overlaps);
        let overlaps = second.overlaps(&first);
        assert!(overlaps);
    }

    #[test]
    fn test_overlaps_outside() {
        let first = Rectangle::new(5, 5, 5, 5);
        let second = Rectangle::new(0, 0, 20, 20);
        let overlaps = first.overlaps(&second);
        assert!(overlaps);
        let overlaps = second.overlaps(&first);
        assert!(overlaps);
    }

    #[test]
    fn test_overlaps_equals() {
        let first = Rectangle::new(5, 5, 5, 5);
        let overlaps = first.overlaps(&first);
        assert!(overlaps);
        let overlaps = first.overlaps(&first);
        assert!(overlaps);
    }

    #[test]
    fn test_overlaps_above() {
        let first = Rectangle::new(0, 0, 5, 5);
        let second = Rectangle::new(5, 5, 5, 5);
        let overlaps = first.overlaps(&second);
        assert!(!overlaps);
        let overlaps = second.overlaps(&first);
        assert!(!overlaps);
    }

    #[test]
    fn test_overlaps_below() {
        let first = Rectangle::new(5, 5, 5, 5);
        let second = Rectangle::new(0, 0, 5, 5);
        let overlaps = first.overlaps(&second);
        assert!(!overlaps);
        let overlaps = second.overlaps(&first);
        assert!(!overlaps);
    }

    #[test]
    fn test_overlaps_to_the_left() {
        let first = Rectangle::new(0, 0, 5, 5);
        let second = Rectangle::new(0, 5, 5, 5);
        let overlaps = first.overlaps(&second);
        assert!(!overlaps);
        let overlaps = second.overlaps(&first);
        assert!(!overlaps);
    }

    #[test]
    fn test_overlaps_to_the_right() {
        let first = Rectangle::new(0, 5, 5, 5);
        let second = Rectangle::new(0, 0, 5, 5);
        let overlaps = first.overlaps(&second);
        assert!(!overlaps);
        let overlaps = second.overlaps(&first);
        assert!(!overlaps);
    }

    #[test]
    fn test_move_to_left() {
        let rectangle = Rectangle::new(0, 5, 10, 10);
        let moved = rectangle.moved_to(0, 0);
        assert_eq!(0, moved.bottom());
        assert_eq!(0, moved.left());
        // The width and height don't change
        assert_eq!(10, moved.height());
        assert_eq!(10, moved.width());
    }

    #[test]
    fn test_move_to_right() {
        let rectangle = Rectangle::new(0, 0, 10, 10);
        let moved = rectangle.moved_to(5, 0);
        assert_eq!(0, moved.bottom());
        assert_eq!(5, moved.left());
        // The width and height don't change
        assert_eq!(10, moved.height());
        assert_eq!(10, moved.width());
    }

    #[test]
    fn test_move_to_down() {
        let rectangle = Rectangle::new(5, 0, 10, 10);
        let moved = rectangle.moved_to(0, 0);
        assert_eq!(0, moved.bottom());
        assert_eq!(0, moved.left());
        // The width and height don't change
        assert_eq!(10, moved.height());
        assert_eq!(10, moved.width());
    }

    #[test]
    fn test_move_to_up() {
        let rectangle = Rectangle::new(0, 0, 10, 10);
        let moved = rectangle.moved_to(0, 5);
        assert_eq!(5, moved.bottom());
        assert_eq!(0, moved.left());
        // The width and height don't change
        assert_eq!(10, moved.height());
        assert_eq!(10, moved.width());
    }
}
