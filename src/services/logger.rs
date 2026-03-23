pub trait Logger {
    fn info(&self, msg: &str);
    fn error(&self, msg: &str);
}
