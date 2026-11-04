use std::ops::{Add, Div, Mul, Sub};

use crate::{point_2d::Point2d, rectangle::Rectangle, vector_2d::Vector2d};

#[derive(Debug, Clone, Copy)]
pub struct LineSegment<T> {
    start: Point2d<T>,
    end: Point2d<T>,
}

impl<T> LineSegment<T> {
    #[inline]
    #[must_use]
    pub fn from_points(start: Point2d<T>, end: Point2d<T>) -> Self {
        Self { start, end }
    }

    #[inline]
    #[must_use]
    pub fn start(self) -> Point2d<T> {
        self.start
    }

    #[inline]
    #[must_use]
    pub fn end(self) -> Point2d<T> {
        self.end
    }
}

impl<T> LineSegment<T>
where
    T: Copy + Add<Output = T>,
{
    #[inline]
    #[must_use]
    pub fn from_point_vector(start: Point2d<T>, vector: Vector2d<T>) -> Self {
        let end = start + vector;
        Self { start, end }
    }
}

impl<T, S> LineSegment<T>
where
    T: Copy + Sub<Output = S>,
{
    #[inline]
    #[must_use]
    pub fn difference(&self) -> Vector2d<S> {
        self.end - self.start
    }
}

impl LineSegment<f32> {
    #[inline]
    #[must_use]
    pub fn length(&self) -> f32 {
        self.difference().length()
    }
}

impl<T> LineSegment<T>
where
    T: Copy + Sub,
    <T as Sub<T>>::Output: PartialEq<<T as Sub<T>>::Output> + Default + Div,
{
    pub fn slope(&self) -> Option<<<T as Sub<T>>::Output as Div<<T as Sub<T>>::Output>>::Output> {
        let run = self.end.x() - self.start.x();
        if run == <T as Sub<T>>::Output::default() {
            return None;
        }
        let rise = self.end.y() - self.start.y();
        Some(rise / run)
    }
}

impl<T, S> LineSegment<T>
where
    T: Copy + PartialOrd + Default + Sub<Output = T> + Div<Output = S> + Add<Output = T>,
    S: Copy + PartialOrd + Mul<T, Output = T>,
{
    pub fn intersects_rectangle(&self, rectangle: &Rectangle<T>) -> bool {
        if self.start.intersects_rectangle(rectangle) || self.end.intersects_rectangle(rectangle) {
            // One of the points is within the rectangle.
            return true;
        }
        // Now we know that the start and end points of the line segment are outside of the
        // rectangle. The only way they intersect is if the segment passes through. Like a point,
        // the segment counts as inside along the rectangle's left and bottom sides but not along
        // its right and top sides.
        let Some(slope) = self.slope() else {
            // The line segment is vertical, thus the x coordinate is the same for both the start
            // and end point. It intersects if that x coordinate is between the rectangle's left
            // and right sides and its span of y coordinates overlaps the rectangle's.
            let x = self.start.x();
            if x < rectangle.left() || x >= rectangle.right() {
                return false;
            }
            return Self::overlaps_vertically(self.start.y(), self.end.y(), rectangle);
        };
        // Only the part of the segment whose x coordinates lie between the rectangle's left and
        // right sides can pass through it, so clamp the segment's x span to the rectangle's.
        let (min_x, max_x) = ordered(self.start.x(), self.end.x());
        let low_x = if min_x > rectangle.left() {
            min_x
        } else {
            rectangle.left()
        };
        let high_x = if max_x < rectangle.right() {
            max_x
        } else {
            rectangle.right()
        };
        // When the spans share at most a single x coordinate, the segment only touches the
        // rectangle at one of its own end points, which we know are outside, or along the right
        // side, which the rectangle excludes.
        if low_x >= high_x {
            return false;
        }
        // Over the clamped span, the segment's y coordinates run between its y coordinates at
        // either end, because the segment is straight. It passes through when that run overlaps
        // the rectangle's span of y coordinates.
        let y_at = |x: T| self.start.y() + slope * (x - self.start.x());
        let left_y = y_at(low_x);
        let right_y = y_at(high_x);
        if high_x == rectangle.right() && right_y > left_y {
            // The segment rises toward the excluded right side, so its highest y is never
            // reached inside the rectangle. It must climb strictly above the bottom before
            // getting there, or else it only touches the bottom-right corner.
            return left_y < rectangle.top() && right_y > rectangle.bottom();
        }
        Self::overlaps_vertically(left_y, right_y, rectangle)
    }

    #[inline]
    #[must_use]
    fn overlaps_vertically(y_1: T, y_2: T, rectangle: &Rectangle<T>) -> bool {
        let (low_y, high_y) = ordered(y_1, y_2);
        low_y < rectangle.top() && high_y >= rectangle.bottom()
    }
}

#[inline]
#[must_use]
fn ordered<T>(first: T, second: T) -> (T, T)
where
    T: PartialOrd,
{
    if first <= second {
        (first, second)
    } else {
        (second, first)
    }
}

#[cfg(test)]
mod tests {
    use uom::si::length::meter;

    use crate::{
        line_segment::LineSegment, point_2d::Point2d, rectangle::Rectangle,
        units::si::length::Length,
    };

    #[test]
    fn slope_of_horizontal_line_zero() {
        let start_point = Point2d::<f32>::from_x_y(1.0, 10.0);
        let end_point = Point2d::<f32>::from_x_y(5.0, 10.0);
        let line = LineSegment::from_points(start_point, end_point);
        let slope = line.slope();
        assert!(matches!(slope, Some(0.0)));
    }

    #[test]
    fn slope_of_vertical_line_none() {
        let start_point = Point2d::<f32>::from_x_y(1.0, 1.0);
        let end_point = Point2d::<f32>::from_x_y(1.0, 10.0);
        let line = LineSegment::from_points(start_point, end_point);
        let slope = line.slope();
        assert!(slope.is_none());
    }

    #[test]
    fn slope_of_ascending_line_positive() {
        let start_point = Point2d::<f32>::from_x_y(1.0, 1.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 10.0);
        let line = LineSegment::from_points(start_point, end_point);
        let slope = line.slope();
        assert!(matches!(slope, Some(1.0)));
    }

    #[test]
    fn slope_of_ascending_line_negative() {
        let start_point = Point2d::<f32>::from_x_y(1.0, 10.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 1.0);
        let line = LineSegment::from_points(start_point, end_point);
        let slope = line.slope();
        assert!(matches!(slope, Some(-1.0)));
    }

    #[test]
    fn intersects_rectangle_diagonal() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 0.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 10.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_when_both_points_inside() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(4.0, 4.0);
        let end_point = Point2d::<f32>::from_x_y(5.0, 5.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_when_start_inside() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(4.0, 4.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 10.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_when_end_inside() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 0.0);
        let end_point = Point2d::<f32>::from_x_y(4.0, 4.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_vertical_down_middle() {
        let rectangle = Rectangle::<f32>::new(2.0, 2.0, 4.0, 4.0);
        let start_point = Point2d::<f32>::from_x_y(4.0, 10.0);
        let end_point = Point2d::<f32>::from_x_y(4.0, 0.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn vertical_above_will_not_intersect_rectangle() {
        let rectangle = Rectangle::<f32>::new(2.0, 2.0, 4.0, 4.0);
        let start_point = Point2d::<f32>::from_x_y(6.0, 6.0);
        let end_point = Point2d::<f32>::from_x_y(6.0, 10.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(!result);
    }

    #[test]
    fn vertical_below_will_not_intersect_rectangle() {
        let rectangle = Rectangle::<f32>::new(2.0, 2.0, 4.0, 4.0);
        let start_point = Point2d::<f32>::from_x_y(6.0, 1.9);
        let end_point = Point2d::<f32>::from_x_y(6.0, 0.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(!result);
    }

    #[test]
    fn intersects_rectangle_negative_diagonal() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 9.0);
        let end_point = Point2d::<f32>::from_x_y(9.0, 0.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_bottom_left_corner() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(2.0, 4.0);
        let end_point = Point2d::<f32>::from_x_y(4.0, 4.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_uom_lengths() {
        let meters = |value: f32| Length::new::<meter>(value);
        let rectangle = Rectangle::new(meters(3.0), meters(3.0), meters(3.0), meters(3.0));
        let start_point = Point2d::from_x_y(meters(0.0), meters(9.0));
        let end_point = Point2d::from_x_y(meters(9.0), meters(0.0));
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn segment_short_of_rectangle_will_not_intersect_even_if_line_would() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 0.0);
        let end_point = Point2d::<f32>::from_x_y(2.0, 2.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(!result);
    }

    #[test]
    fn intersects_rectangle_steep_through_top_and_bottom() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(4.0, 10.0);
        let end_point = Point2d::<f32>::from_x_y(5.0, 0.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_vertical_starting_on_top_edge() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(4.0, 6.0);
        let end_point = Point2d::<f32>::from_x_y(4.0, 0.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn horizontal_along_top_edge_will_not_intersect_rectangle() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 6.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 6.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(!result);
    }

    #[test]
    fn intersects_rectangle_horizontal_along_bottom_edge() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 3.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 3.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn rising_through_bottom_right_corner_will_not_intersect_rectangle() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(5.0, 2.0);
        let end_point = Point2d::<f32>::from_x_y(7.0, 4.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(!result);
    }

    #[test]
    fn intersects_rectangle_rising_through_bottom_and_out_right_side() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(4.0, 2.0);
        let end_point = Point2d::<f32>::from_x_y(7.0, 5.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }

    #[test]
    fn intersects_rectangle_horizontal() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 4.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 4.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }
}
