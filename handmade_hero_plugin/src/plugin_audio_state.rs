use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAudioState {
    theta: f32,
}

impl PluginAudioState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self { theta: 0f32 }
    }

    #[inline]
    #[must_use]
    pub fn theta(&self) -> f32 {
        self.theta
    }

    #[inline]
    pub fn set_theta(&mut self, value: f32) {
        self.theta = value;
    }
}
