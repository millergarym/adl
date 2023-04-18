use crate::adlgen::sys::adlast2 as adlast;

pub mod annotations;
pub mod checks;
pub mod loader;
pub mod primitives;
pub mod resolver;
pub mod writer;

pub trait ErrorConsumer {
    fn consume_error(&mut self, error: String) -> ();
}

pub struct ErrorLogger {
    pub failed: bool,
}

impl ErrorLogger {
    pub fn new() -> Self {
        ErrorLogger { failed: false }
    }
}

impl ErrorConsumer for ErrorLogger {
    fn consume_error(&mut self, error: String) {
        log::error!("{}", error);
        self.failed = true;
    }
}
