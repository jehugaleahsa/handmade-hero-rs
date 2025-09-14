#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct StereoSample {
    left: i16,
    right: i16,
}

impl StereoSample {
    pub fn from_left_right(left: i16, right: i16) -> Self {
        Self { left, right }
    }
}
