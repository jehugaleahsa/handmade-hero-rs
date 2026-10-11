pub mod si {
    pub mod length {
        use uom::si::SI;

        pub type Length = uom::si::length::Length<SI<f32>, f32>;
    }

    pub mod linear_number_density {
        use uom::si::SI;

        /// A count per unit of length, such as how many screen pixels span one meter of the
        /// world. Multiplying a [`Length`](super::length::Length) by one leaves a dimensionless
        /// count.
        pub type LinearNumberDensity =
            uom::si::linear_number_density::LinearNumberDensity<SI<f32>, f32>;
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
