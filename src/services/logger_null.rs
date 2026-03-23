use super::Logger;

pub struct NullLogger;

impl Logger for NullLogger {
    fn info(&self, _msg: &str) {}
    fn error(&self, _msg: &str) {}
}
