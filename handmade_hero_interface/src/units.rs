pub mod si {
    pub mod length {
        use uom::si::SI;

        pub type Length = uom::si::length::Length<SI<f32>, f32>;

        unit! {
            system: uom::si;
            quantity: uom::si::length;
            @pixel: 1.0 / 42.85; "px", "pixel", "pixels";
        }
    }

    pub mod time {
        use uom::si::SI;

        pub type Time = uom::si::time::Time<SI<f32>, f32>;
    }

    pub mod frequency {
        use uom::si::SI;

        /// A count of cycles per second, such as the monitor refresh rate or the rate the audio
        /// device consumes samples. Integer storage keeps the audio buffer math exact.
        pub type Frequency = uom::si::frequency::Frequency<SI<u32>, u32>;
    }

    pub mod information {
        use uom::si::SI;

        /// A count of bytes, such as a sound buffer's length or a cursor's offset into it.
        pub type Information = uom::si::information::Information<SI<u32>, u32>;
    }

    pub mod information_rate {
        use uom::si::SI;

        /// A count of bytes per second, such as the rate the audio device drains the sound
        /// buffer. Dividing one by a [`Frequency`](super::frequency::Frequency) cancels the
        /// per-second term and leaves an [`Information`](super::information::Information).
        pub type InformationRate = uom::si::information_rate::InformationRate<SI<u32>, u32>;
    }
}

#[cfg(test)]
mod tests {
    use crate::units::si::length::{Length, pixel};
    use uom::si::length::meter;

    const PIXELS_PER_METER: f32 = 42.85;

    #[test]
    fn test_meter_to_pixels() {
        let meters = Length::new::<meter>(1f32);
        let pixels = meters.get::<pixel>();
        #[expect(clippy::float_cmp)]
        {
            assert_eq!(PIXELS_PER_METER, pixels);
        }
    }

    #[test]
    fn test_pixels_to_meters() {
        let meters = Length::new::<pixel>(PIXELS_PER_METER);
        let pixels = meters.get::<meter>();
        #[expect(clippy::float_cmp)]
        {
            assert_eq!(1.0, pixels);
        }
    }
}
