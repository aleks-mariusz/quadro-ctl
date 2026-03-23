use super::Logger;

pub struct StandardLogger;

impl Logger for StandardLogger {
    fn info(&self, msg: &str) {
        println!("{}", msg);
    }

    fn error(&self, msg: &str) {
        eprintln!("{}", msg);
    }
}
