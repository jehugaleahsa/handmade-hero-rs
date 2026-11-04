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
        // Now we know that the start and end points of the line segement are outside
        // of the rectangle. The only way they intersect is if the lines passes through.
        if let Some(slope) = self.slope() {
            // The line is diagonal or horizontal. For the equation `y = mx + b`, b is where
            // the line segment would intersect the y coordinate. With that, we can calculate
            // the value of y at any given x using the formula. If we plug in the x coordinates
            // of the left and right of the rectangle, we can check if y is between the bottom
            // and right of the rectangle, telling us if they intersect.
            let y_intersect = self.start.y() - slope * self.start.x();
            let y_1 = slope * rectangle.left() + y_intersect;
            if y_1 >= rectangle.bottom() && y_1 < rectangle.top() {
                return true;
            }
            let y_2 = slope * rectangle.right() + y_intersect;
            if y_2 >= rectangle.bottom() && y_2 < rectangle.top() {
                return true;
            }
            return false;
        }
        // The line segment is vertical, thus the x coordinate is the same for both
        // the start and end point. It intersects if it's x coordinate is within
        // the rectangle's left and right sides and if the y coordinate intersects
        // the rectangle's bottom or top.
        if self.start.x() < rectangle.left() || self.start.x() >= rectangle.right() {
            return false;
        }
        // Since the start and end can't be inside the rectangle at this point, the start
        // must be above the top or below the bottom, and the opposite for the end.
        if self.start.y() > rectangle.top() {
            self.end.y() < rectangle.bottom()
        } else if self.start.y() < rectangle.bottom() {
            self.end.y() > rectangle.top()
        } else {
            false
        }
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
    fn intersects_rectangle_horizontal() {
        let rectangle = Rectangle::<f32>::new(3.0, 3.0, 3.0, 3.0);
        let start_point = Point2d::<f32>::from_x_y(0.0, 4.0);
        let end_point = Point2d::<f32>::from_x_y(10.0, 4.0);
        let line = LineSegment::from_points(start_point, end_point);
        let result = line.intersects_rectangle(&rectangle);
        assert!(result);
    }
}
