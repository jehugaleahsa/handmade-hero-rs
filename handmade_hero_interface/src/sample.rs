use crate::units::si::information::Information;

pub trait Sample: Copy {
    fn channel_size(&self) -> Information;
    fn channel_count(&self) -> u16;
}
