use std::time::Instant;

/// Prints per-frame timing to stderr, the way Casey prints it to the debugger output:
/// milliseconds per frame, frames per second, and CPU megacycles per frame.
///
/// Each frame is measured from the end of the previous frame to the end of this one, so the
/// numbers include the time spent waiting for the target frame rate and flipping the buffer.
#[derive(Debug)]
pub struct Win32FrameStats {
    last_instant: Instant,
    last_cycles: u64,
}

impl Win32FrameStats {
    #[must_use]
    pub fn start() -> Self {
        Self {
            last_instant: Instant::now(),
            last_cycles: read_cycle_counter(),
        }
    }

    pub fn end_frame(&mut self) {
        let end_instant = Instant::now();
        let end_cycles = read_cycle_counter();

        let elapsed = end_instant.duration_since(self.last_instant);
        let milliseconds_per_frame = elapsed.as_secs_f64() * 1000.0;
        let frames_per_second = 1.0 / elapsed.as_secs_f64();
        #[expect(clippy::cast_precision_loss)]
        let megacycles_per_frame = end_cycles.wrapping_sub(self.last_cycles) as f64 / 1_000_000.0;
        eprintln!(
            "{milliseconds_per_frame:.2}ms/f, {frames_per_second:.2}f/s, {megacycles_per_frame:.2}mc/f"
        );

        self.last_instant = end_instant;
        self.last_cycles = end_cycles;
    }
}

/// The CPU's time stamp counter, which ticks at a fixed rate near the processor's base clock.
#[cfg(target_arch = "x86_64")]
fn read_cycle_counter() -> u64 {
    // SAFETY: RDTSC is available on every x86-64 processor and only reads a counter.
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Architectures without a time stamp counter report zero cycles.
#[cfg(not(target_arch = "x86_64"))]
fn read_cycle_counter() -> u64 {
    0
}
