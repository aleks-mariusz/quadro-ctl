use std::time::Duration;

use super::Sleeper;

pub struct NoopSleeper;

impl Sleeper for NoopSleeper {
    fn sleep(&self, _duration: Duration) {}
}
