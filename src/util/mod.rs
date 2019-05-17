use ::std::time::{Duration, Instant};
use enumset::{EnumSet, EnumSetType};

#[derive(Debug, EnumSetType)]
#[enumset(serialize_repr = "u8")]
pub enum Face {
    Back = 0,
    Front = 1,
    Right = 2,
    Left = 3,
    Top = 4,
    Bottom = 5,
}

impl Face {
    pub fn flip(self) -> Face {
        match (self as u8) ^ 1 {
            0 => Face::Back,
            1 => Face::Front,
            2 => Face::Right,
            3 => Face::Left,
            4 => Face::Top,
            5 => Face::Bottom,
            _ => unreachable!()
        }
    }
}

pub type Faces = EnumSet<Face>;

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
