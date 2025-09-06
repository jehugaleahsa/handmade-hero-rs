use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct PerformanceMetrics {
    elapsed_time: Duration,
    elapsed_cycles: u64,
}

impl PerformanceMetrics {
    #[inline]
    #[must_use]
    pub fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }

    #[inline]
    #[must_use]
    pub fn elapsed_cycles(&self) -> u64 {
        self.elapsed_cycles
    }
}

#[derive(Debug)]
pub struct PerformanceCounter {
    last_instant: Instant,
    last_cycle_count: u64,
}

impl PerformanceCounter {
    pub fn start() -> Self {
        let last_instant = Instant::now();
        let last_cycle_count = Self::query_cycle_count();
        Self {
            last_instant,
            last_cycle_count,
        }
    }

    pub fn restart(&mut self) -> PerformanceMetrics {
        let current = Instant::now();
        let current_cycle_count = Self::query_cycle_count();
        let elapsed_time = current.duration_since(self.last_instant);
        let elapsed_cycles = current_cycle_count - self.last_cycle_count;
        let metrics = PerformanceMetrics {
            elapsed_time,
            elapsed_cycles,
        };
        self.last_instant = current;
        self.last_cycle_count = current_cycle_count;
        metrics
    }

    #[must_use]
    #[inline]
    fn query_cycle_count() -> u64 {
        #[cfg(target_arch = "x86")]
        unsafe {
            core::arch::x86::_rdtsc()
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::x86_64::_rdtsc()
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        0u64
    }
}
