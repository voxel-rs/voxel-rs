use ::std::time::{Duration, Instant};

pub mod lazy_container;

pub struct Ticker {
    tick_duration: Duration,
    last_tick: Instant,
}

impl Ticker {
    pub fn from_tick_duration(tick_duration: Duration) -> Self {
        Self {
            tick_duration,
            last_tick: Instant::now(),
        }
    }

    /// The tick rate is the amount of ticks per sec (must not be 0)
    pub fn from_tick_rate(tick_rate: u32) -> Self {
        assert!(tick_rate != 0);
        Self::from_tick_duration(Duration::new(0, 1_000_000_000 / tick_rate))
    }

    pub fn try_tick(&mut self) -> bool {
        let current_time = Instant::now();
        let elapsed_time = current_time - self.last_tick;
        if elapsed_time >= self.tick_duration {
            self.last_tick = current_time;
            true
        } else {
            false
        }
    }
}
