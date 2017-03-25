extern crate libc;
use std::io;
use std::mem;

pub const SHUT_READ: usize = 0;
pub const SHUT_WRITE: usize = 1;

pub struct Shutdown([libc::c_int; 2]);

impl Shutdown {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let mut pipe: [libc::c_int; 2] = [0, 0];
            if libc::pipe2(&mut pipe[0], libc::O_NONBLOCK) < 0 {
                let errno = *libc::__errno_location();
                assert!(errno == libc::EMFILE || errno == libc::ENFILE);
                Err(io::Error::new(io::ErrorKind::Other, "no resource"))
            } else {
                Ok(Shutdown(pipe))
            }
        }
    }

    pub fn shutdown(&self, read_or_write: usize) {
        assert!(read_or_write == SHUT_READ || read_or_write == SHUT_WRITE);
        unsafe {
            libc::close(self.0[read_or_write]);
        }
    }

    pub fn shuted(&self, read_or_write: usize) -> bool {
        assert!(read_or_write == SHUT_READ || read_or_write == SHUT_WRITE);
        unsafe {
            if read_or_write == SHUT_READ {
                let data: [u8; 1] = [0];
                let data: *const libc::c_void = mem::transmute(&data[0]);
                if libc::write(self.0[1], data, 1) < 0 {
                    let errno = *libc::__errno_location();
                    if errno == libc::EPIPE {
                        return true;
                    }
                }
                return false;
            } else {
                let mut data: [u8; 4096] = [0; 4096];
                let data: *mut libc::c_void = mem::transmute(&mut data[0]);
                if libc::read(self.0[0], data, 4096) == 0 {
                    return true;
                }
                return false;
            }
        }
    }
}

impl Drop for Shutdown {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0[0]);
            libc::close(self.0[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown() {
        let s = Shutdown::new().unwrap();
        assert_eq!(s.shuted(SHUT_READ), false);
        assert_eq!(s.shuted(SHUT_WRITE), false);

        let s = Shutdown::new().unwrap();
        s.shutdown(SHUT_READ);
        assert_eq!(s.shuted(SHUT_READ), true);

        let s = Shutdown::new().unwrap();
        s.shutdown(SHUT_WRITE);
        assert_eq!(s.shuted(SHUT_WRITE), true);
    }
}
