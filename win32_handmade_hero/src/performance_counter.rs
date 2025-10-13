use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct PerformanceMetrics {
    elapsed_time: Duration,
}

impl PerformanceMetrics {
    #[inline]
    #[must_use]
    pub fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }
}

#[derive(Debug)]
pub struct PerformanceCounter {
    last_instant: Instant,
}

impl PerformanceCounter {
    pub fn start() -> Self {
        let last_instant = Instant::now();
        Self { last_instant }
    }

    pub fn metrics(&self) -> PerformanceMetrics {
        let current = Instant::now();
        let elapsed_time = current.duration_since(self.last_instant);
        PerformanceMetrics { elapsed_time }
    }

    pub fn restart(&mut self) {
        self.last_instant = Instant::now();
    }
}
