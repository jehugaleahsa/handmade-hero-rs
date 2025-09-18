#[derive(Debug, Default, Copy, Clone)]
pub struct JoystickTransition {
    start: f32,
    min: f32,
    max: f32,
    end: f32,
}

impl JoystickTransition {
    #[inline]
    #[must_use]
    pub fn start(&self) -> f32 {
        self.start
    }

    #[inline]
    pub fn set_start(&mut self, value: f32) {
        self.start = value;
    }

    #[inline]
    #[must_use]
    pub fn min(&self) -> f32 {
        self.min
    }

    #[inline]
    pub fn set_min(&mut self, value: f32) {
        self.min = value;
    }

    #[inline]
    #[must_use]
    pub fn max(&self) -> f32 {
        self.max
    }

    #[inline]
    pub fn set_max(&mut self, value: f32) {
        self.max = value;
    }

    #[inline]
    #[must_use]
    pub fn end(&self) -> f32 {
        self.end
    }

    #[inline]
    pub fn set_end(&mut self, value: f32) {
        self.end = value;
    }
}
