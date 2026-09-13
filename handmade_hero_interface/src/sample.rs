use crate::units::si::information::Information;

pub trait Sample: Copy {
    #[must_use]
    fn channel_size(&self) -> Information;
    #[must_use]
    fn channel_count(&self) -> u16;
}
