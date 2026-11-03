use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

use crate::{rectangle::Rectangle, vector_2d::Vector2d};

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
pub struct Point2d<T> {
    x: T,
    y: T,
}

impl<T> Point2d<T>
where
    T: Copy,
{
    #[inline]
    #[must_use]
    pub fn from_x_y(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    pub fn x(self) -> T {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(self) -> T {
        self.y
    }

    #[inline]
    #[must_use]
    pub fn to_tuple(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T> Add<Vector2d<T>> for Point2d<T>
where
    T: Copy + Add<Output = T>,
{
    type Output = Self;

    #[inline]
    fn add(self, vector: Vector2d<T>) -> Self {
        let x = self.x + vector.x();
        let y = self.y + vector.y();
        Self::from_x_y(x, y)
    }
}

impl<T> Add<Point2d<T>> for Vector2d<T>
where
    T: Copy + Add<Output = T>,
{
    type Output = Point2d<T>;

    #[inline]
    fn add(self, point: Point2d<T>) -> Self::Output {
        let x = self.x() + point.x;
        let y = self.y() + point.y;
        Self::Output::from_x_y(x, y)
    }
}

impl<T, S> Sub for Point2d<T>
where
    T: Sub<Output = S>,
{
    type Output = Vector2d<S>;

    #[inline]
    fn sub(self, other: Self) -> Self::Output {
        let x = self.x - other.x;
        let y = self.y - other.y;
        Self::Output::from_x_y(x, y)
    }
}

impl<T> PartialEq for Point2d<T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T> Eq for Point2d<T> where T: Eq {}

impl<T> Point2d<T>
where
    T: Copy + PartialOrd<T>,
{
    #[inline]
    #[must_use]
    pub fn intersects_rectangle(&self, rectangle: &Rectangle<T>) -> bool {
        self.x >= rectangle.left()
            && self.x < rectangle.right()
            && self.y >= rectangle.bottom()
            && self.y < rectangle.top()
    }
}

impl From<Point2d<u8>> for Point2d<f32> {
    fn from(value: Point2d<u8>) -> Self {
        let x = f32::from(value.x());
        let y = f32::from(value.y());
        Self::from_x_y(x, y)
    }
}

#[cfg(test)]
mod tests {
    use crate::{point_2d::Point2d, rectangle::Rectangle};

    #[test]
    fn intersects_rectangle_if_inside() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let point = Point2d::<f32>::from_x_y(4.0, 4.0);
        let result = point.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_if_on_bottom_edge() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let point = Point2d::<f32>::from_x_y(4.0, 3.0);
        let result = point.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_if_on_left_edge() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let point = Point2d::<f32>::from_x_y(3.0, 4.0);
        let result = point.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn will_not_intersect_rectangle_if_on_top_edge() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let point = Point2d::<f32>::from_x_y(4.0, 6.0);
        let result = point.intersects_rectangle(&rectangle);
        assert!(!result);
    }

    #[test]
    fn will_not_intersect_rectangle_if_on_right_edge() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let point = Point2d::<f32>::from_x_y(6.0, 4.0);
        let result = point.intersects_rectangle(&rectangle);
        assert!(!result);
    }
}
