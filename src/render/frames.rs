use std::collections::LinkedList;
use std;

pub struct FrameCounter {
    frames: LinkedList<f64>,
    timer: std::time::SystemTime,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            frames: LinkedList::new(),
            timer: std::time::SystemTime::now(),
        }
    }

    pub fn frame(&mut self) -> usize {
        let dur = self.timer.elapsed().unwrap();
        let ts = (dur.subsec_nanos() as f64) + (dur.as_secs() as f64) * 1e9;
        while self.frames.len() > 0 && ts - self.frames.front().unwrap() > 1e9 {
            self.frames.pop_front();
        }
        self.frames.push_back(ts);
        self.frames.len()
    }
}