use std::time::Duration;

pub trait Sleeper {
    fn sleep(&self, duration: Duration);
}
