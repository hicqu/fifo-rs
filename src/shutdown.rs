use std::sync::atomic::{AtomicUsize, Ordering};

pub const SHUT_READ: usize = 1;
pub const SHUT_WRITE: usize = 2;

pub struct Shutdown(AtomicUsize);

impl Shutdown {
    pub fn new() -> Self {
        Shutdown(AtomicUsize::new(0))
    }

    pub fn shutdown(&self, read_or_write: usize) {
        assert!(read_or_write == SHUT_READ || read_or_write == SHUT_WRITE);
        let flags = self.0.load(Ordering::Acquire);
        self.0.store(flags | read_or_write, Ordering::Release);
    }

    pub fn shuted(&self, read_or_write: usize) -> bool {
        assert!(read_or_write == SHUT_READ || read_or_write == SHUT_WRITE);
        self.0.load(Ordering::Acquire) & read_or_write != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown() {
        let s = Shutdown::new();
        assert_eq!(s.shuted(SHUT_READ), false);
        assert_eq!(s.shuted(SHUT_WRITE), false);

        let s = Shutdown::new();
        s.shutdown(SHUT_READ);
        assert_eq!(s.shuted(SHUT_READ), true);

        let s = Shutdown::new();
        s.shutdown(SHUT_WRITE);
        assert_eq!(s.shuted(SHUT_WRITE), true);
    }
}
