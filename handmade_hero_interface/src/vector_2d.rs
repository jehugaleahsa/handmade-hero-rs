use std::ops::{Add, Div, Mul, Neg, Sub};

use uom::{
    Conversion,
    num_traits::Num,
    si::{Dimension, Quantity, Units},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector2d<T> {
    values: [T; 2],
}

impl<T> Vector2d<T> {
    #[inline]
    #[must_use]
    pub fn from_x_y(x: T, y: T) -> Self {
        let values = [x, y];
        Self { values }
    }

    #[inline]
    #[must_use]
    pub fn values(self) -> [T; 2] {
        self.values
    }
}

impl<T> Vector2d<T>
where
    T: Copy,
{
    #[inline]
    #[must_use]
    pub fn splat(value: T) -> Self {
        Self::from_x_y(value, value)
    }

    #[inline]
    #[must_use]
    pub fn x(self) -> T {
        self.values[0]
    }

    #[inline]
    #[must_use]
    pub fn y(self) -> T {
        self.values[1]
    }
}

impl<T> Add for Vector2d<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        let x = self.x() + other.x();
        let y = self.y() + other.y();
        Self::from_x_y(x, y)
    }
}

impl<T> Neg for Vector2d<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        let x = -self.x();
        let y = -self.y();
        Self::from_x_y(x, y)
    }
}

impl<T> Sub for Vector2d<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        let x = self.x() - other.x();
        let y = self.y() - other.y();
        Self::from_x_y(x, y)
    }
}

impl<T> Vector2d<T>
where
    T: Mul<Output = T> + Copy,
{
    #[inline]
    #[must_use]
    pub fn hadamard(self, other: Self) -> Self {
        let x = self.x() * other.x();
        let y = self.y() * other.y();
        Self::from_x_y(x, y)
    }
}

// The scalar can be any type the components multiply by, so this covers a plain f32 as well as a
// uom quantity, and the components take on whatever type the product has.
impl<T, S> Mul<S> for Vector2d<T>
where
    T: Mul<S> + Copy,
    S: Copy,
{
    type Output = Vector2d<<T as Mul<S>>::Output>;

    #[inline]
    fn mul(self, rhs: S) -> Self::Output {
        let x = self.x() * rhs;
        let y = self.y() * rhs;
        Self::Output::from_x_y(x, y)
    }
}

// Like multiplication, the scalar can be a plain f32 or a uom quantity, and the components take on
// whatever type the quotient has.
impl<T, S> Div<S> for Vector2d<T>
where
    T: Div<S> + Copy,
    S: Copy,
{
    type Output = Vector2d<<T as Div<S>>::Output>;

    #[inline]
    fn div(self, rhs: S) -> Self::Output {
        let x = self.x() / rhs;
        let y = self.y() / rhs;
        Self::Output::from_x_y(x, y)
    }
}

impl Mul<Vector2d<f32>> for f32 {
    type Output = Vector2d<f32>;

    #[inline]
    fn mul(self, rhs: Vector2d<f32>) -> Self::Output {
        let x = self * rhs.x();
        let y = self * rhs.y();
        Self::Output::from_x_y(x, y)
    }
}

impl Mul<Vector2d<f64>> for f64 {
    type Output = Vector2d<f64>;

    #[inline]
    fn mul(self, rhs: Vector2d<f64>) -> Self::Output {
        let x = self * rhs.x();
        let y = self * rhs.y();
        Self::Output::from_x_y(x, y)
    }
}

// The components take on whatever dimension uom gives the product: a dimensionless scalar keeps
// the dimension, while a dimensioned one combines with it (Time × Velocity is a Length).
impl<D, U, V, T> Mul<Vector2d<T>> for Quantity<D, U, V>
where
    D: Dimension + ?Sized,
    U: Units<V> + ?Sized,
    V: Num + Conversion<V>,
    Self: Mul<T> + Copy,
    T: Copy,
{
    type Output = Vector2d<<Self as Mul<T>>::Output>;

    #[inline]
    fn mul(self, rhs: Vector2d<T>) -> Self::Output {
        let x = self * rhs.x();
        let y = self * rhs.y();
        Self::Output::from_x_y(x, y)
    }
}

impl<T> Default for Vector2d<T>
where
    T: Default,
{
    #[inline]
    fn default() -> Self {
        let values = Default::default();
        Self { values }
    }
}

impl From<Vector2d<u8>> for Vector2d<f32> {
    #[inline]
    fn from(value: Vector2d<u8>) -> Self {
        Self::from_x_y(f32::from(value.x()), f32::from(value.y()))
    }
}

#[cfg(test)]
mod tests {
    use uom::si::{
        area::square_meter, f32::Ratio, length::meter, ratio::ratio, time::second,
        velocity::meter_per_second,
    };

    use crate::{
        units::si::{length::Length, time::Time},
        vector_2d::Vector2d,
    };

    #[test]
    fn scalar_multiply_uom_ratio_keeps_dimension() {
        let x_length = Length::new::<meter>(2.0f32);
        let y_length = Length::new::<meter>(3.0f32);
        let vector = Vector2d::from_x_y(x_length, y_length);
        let scaled = Ratio::new::<ratio>(2.0f32) * vector;
        assert_eq!(4.0f32, scaled.x().get::<meter>());
        assert_eq!(6.0f32, scaled.y().get::<meter>());
    }

    #[test]
    fn scalar_multiply_uom_length_gives_area() {
        let x_length = Length::new::<meter>(2.0f32);
        let y_length = Length::new::<meter>(3.0f32);
        let vector = Vector2d::from_x_y(x_length, y_length);
        let scaled = Length::new::<meter>(2.0f32) * vector;
        assert_eq!(4.0f32, scaled.x().get::<square_meter>());
        assert_eq!(6.0f32, scaled.y().get::<square_meter>());
    }

    #[test]
    fn scalar_multiply_uom_time_gives_length() {
        let x_velocity = uom::si::f32::Velocity::new::<meter_per_second>(2.0f32);
        let y_velocity = uom::si::f32::Velocity::new::<meter_per_second>(3.0f32);
        let vector = Vector2d::from_x_y(x_velocity, y_velocity);
        let moved = Time::new::<second>(2.0f32) * vector;
        assert_eq!(4.0f32, moved.x().get::<meter>());
        assert_eq!(6.0f32, moved.y().get::<meter>());
    }

    #[test]
    fn scalar_multiply_uom_f64() {
        let x_length = uom::si::f64::Length::new::<meter>(2.0f64);
        let y_length = uom::si::f64::Length::new::<meter>(3.0f64);
        let vector = Vector2d::from_x_y(x_length, y_length);
        let scaled = uom::si::f64::Ratio::new::<ratio>(2.0f64) * vector;
        assert_eq!(4.0f64, scaled.x().get::<meter>());
        assert_eq!(6.0f64, scaled.y().get::<meter>());
    }

    #[test]
    fn vector_times_f32_scalar() {
        let vector = Vector2d::from_x_y(2.0f32, 3.0f32);
        let scaled = vector * 2.0f32;
        assert_eq!(4.0f32, scaled.x());
        assert_eq!(6.0f32, scaled.y());
    }

    #[test]
    fn vector_times_uom_ratio_keeps_dimension() {
        let x_length = Length::new::<meter>(2.0f32);
        let y_length = Length::new::<meter>(3.0f32);
        let vector = Vector2d::from_x_y(x_length, y_length);
        let scaled = vector * Ratio::new::<ratio>(2.0f32);
        assert_eq!(4.0f32, scaled.x().get::<meter>());
        assert_eq!(6.0f32, scaled.y().get::<meter>());
    }

    #[test]
    fn vector_times_uom_time_gives_length() {
        let x_velocity = uom::si::f32::Velocity::new::<meter_per_second>(2.0f32);
        let y_velocity = uom::si::f32::Velocity::new::<meter_per_second>(3.0f32);
        let vector = Vector2d::from_x_y(x_velocity, y_velocity);
        let moved = vector * Time::new::<second>(2.0f32);
        assert_eq!(4.0f32, moved.x().get::<meter>());
        assert_eq!(6.0f32, moved.y().get::<meter>());
    }

    #[test]
    fn splat_copies_value_to_both_components() {
        let vector = Vector2d::splat(Length::new::<meter>(2.0f32));
        assert_eq!(2.0f32, vector.x().get::<meter>());
        assert_eq!(2.0f32, vector.y().get::<meter>());
    }

    #[test]
    fn vector_divided_by_f32_scalar() {
        let vector = Vector2d::from_x_y(4.0f32, 6.0f32);
        let scaled = vector / 2.0f32;
        assert_eq!(2.0f32, scaled.x());
        assert_eq!(3.0f32, scaled.y());
    }

    #[test]
    fn vector_divided_by_uom_ratio_keeps_dimension() {
        let x_length = Length::new::<meter>(4.0f32);
        let y_length = Length::new::<meter>(6.0f32);
        let vector = Vector2d::from_x_y(x_length, y_length);
        let scaled = vector / Ratio::new::<ratio>(2.0f32);
        assert_eq!(2.0f32, scaled.x().get::<meter>());
        assert_eq!(3.0f32, scaled.y().get::<meter>());
    }

    #[test]
    fn vector_divided_by_uom_time_gives_velocity() {
        let x_length = Length::new::<meter>(4.0f32);
        let y_length = Length::new::<meter>(6.0f32);
        let vector = Vector2d::from_x_y(x_length, y_length);
        let velocity = vector / Time::new::<second>(2.0f32);
        assert_eq!(2.0f32, velocity.x().get::<meter_per_second>());
        assert_eq!(3.0f32, velocity.y().get::<meter_per_second>());
    }
}
