use super::Logger;

pub struct StandardLogger;

impl Logger for StandardLogger {
    fn info(&self, msg: &str) {
        eprintln!("{}", msg);
    }

    fn error(&self, msg: &str) {
        eprintln!("{}", msg);
    }
}
