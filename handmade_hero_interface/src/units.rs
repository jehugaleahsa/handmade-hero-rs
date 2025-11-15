pub mod si {
    pub mod length {
        use uom::si::SI;

        pub type Length = uom::si::length::Length<SI<f32>, f32>;

        pub(crate) const PIXELS_PER_METER: f32 = 42.85;
        pub(crate) const METERS_PER_PIXEL: f32 = 1.0 / PIXELS_PER_METER;

        unit! {
            system: uom::si;
            quantity: uom::si::length;

            @pixel: V::from(crate::units::si::length::METERS_PER_PIXEL); "px", "pixel", "pixels";
        }
    }

    pub mod time {
        use uom::si::SI;

        pub type Time = uom::si::time::Time<SI<f32>, f32>;
    }
}

#[cfg(test)]
mod tests {
    use crate::units::si::length::{Length, PIXELS_PER_METER, pixel};
    use uom::si::length::meter;

    #[test]
    fn test_meter_to_pixels() {
        let meters = Length::new::<meter>(1f32);
        let pixels = meters.get::<pixel>();
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(PIXELS_PER_METER, pixels);
        }
    }

    #[test]
    fn test_pixels_to_meters() {
        let meters = Length::new::<pixel>(PIXELS_PER_METER);
        let pixels = meters.get::<meter>();
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(1.0, pixels);
        }
    }
}
