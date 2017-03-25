#![feature(alloc, heap_api)]
#![feature(optin_builtin_traits)]
extern crate alloc;
mod shutdown;
use std::{io, slice, thread, time};
use std::cmp::min;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use alloc::heap;
use shutdown::{Shutdown, SHUT_READ, SHUT_WRITE};

struct _Inner {
    buffer: *mut u8,
    size: usize,
    pin: AtomicUsize,
    pout: AtomicUsize,
    shutdown: Shutdown,
}

impl _Inner {
    fn new(size: usize) -> io::Result<Self> {
        let shutdown = Shutdown::new()?;
        let inner = _Inner {
            buffer: unsafe { heap::allocate(size, 1) },
            size: size,
            pin: AtomicUsize::new(0),
            pout: AtomicUsize::new(0),
            shutdown: shutdown,
        };
        Ok(inner)
    }
}

unsafe impl Sync for _Inner {}

impl Drop for _Inner {
    fn drop(&mut self) {
        unsafe { heap::deallocate(self.buffer, self.size, 1); }
    }
}

type Inner = Arc<_Inner>;

/// The fifo sender.
pub struct Sender {
    inner: Inner,
}

/// The fifo receiver.
pub struct Receiver {
    inner: Inner,
}

impl Drop for Sender {
    fn drop(&mut self) {
        self.inner.shutdown.shutdown(SHUT_WRITE);
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.inner.shutdown.shutdown(SHUT_READ);
    }
}

unsafe impl Send for Sender {}
unsafe impl Send for Receiver {}
impl !Sync for Sender {}
impl !Sync for Receiver {}


const FIFO_MIN_CAPACITY: usize = 128;

fn align_up(size: usize) -> usize {
    if size < FIFO_MIN_CAPACITY {
        return FIFO_MIN_CAPACITY;
    }
    let mut capacity = FIFO_MIN_CAPACITY;
    loop {
        if capacity >= size {
            return capacity;
        }
        capacity = capacity << 1;
    }
}

/// Returns the sender and receiver pair. The fifo between
/// them has capacity as align_up(size).
pub fn fifo(size: usize) -> io::Result<(Sender, Receiver)> {
    let size = align_up(size);
    let inner = Arc::new(_Inner::new(size)?);
    let sender = Sender { inner: inner.clone() };
    let receiver = Receiver { inner: inner };
    Ok((sender, receiver))
}

impl io::Write for Sender {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let inner: &mut Inner = &mut self.inner;

        let mut pin: usize;
        let mut pout: usize;
        let mut avaliable: usize;
        loop {
            pin = inner.pin.load(Ordering::Relaxed);
            pout = inner.pout.load(Ordering::Acquire);
            avaliable = min(inner.size - (pin - pout), bytes.len());
            if avaliable > 0 {
                break;
            } else {
                if inner.shutdown.shuted(SHUT_READ) {
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
                }
                thread::sleep(time::Duration::from_millis(10));
            };
        }
        let start_pos_1 = pin & (inner.size - 1);
        let len_to_write_1 = min(inner.size - start_pos_1, avaliable);
        let len_to_write_2 = avaliable - len_to_write_1;
        unsafe {
            let mut dest = slice::from_raw_parts_mut(inner.buffer, inner.size);
            dest[start_pos_1..(start_pos_1+len_to_write_1)].copy_from_slice(&bytes[0..len_to_write_1]);
            dest[0..len_to_write_2].copy_from_slice(&bytes[len_to_write_1..avaliable]);
        }
        inner.pin.store(pin + avaliable, Ordering::Release);
        Ok(avaliable)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for Receiver {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let inner: &mut Inner = &mut self.inner;
        let mut pin: usize;
        let mut pout: usize;
        let mut avaliable: usize;
        loop {
            pin = inner.pin.load(Ordering::Acquire);
            pout = inner.pout.load(Ordering::Relaxed);
            avaliable = min(bytes.len(), pin - pout);
            if avaliable > 0 {
                break;
            } else {
                if inner.shutdown.shuted(SHUT_WRITE) {
                    return Ok(0);
                }
                thread::sleep(time::Duration::from_millis(10));
            }
        }
        let start_pos_1 = pout & (inner.size - 1);
        let len_to_read_1 = min(inner.size - start_pos_1, avaliable);
        let len_to_read_2 = avaliable - len_to_read_1;
        unsafe {
            let src = slice::from_raw_parts_mut(inner.buffer, inner.size);
            bytes[0..len_to_read_1].copy_from_slice(&src[start_pos_1..(start_pos_1+len_to_read_1)]); 
            bytes[len_to_read_1..avaliable].copy_from_slice(&src[0..len_to_read_2]);
        }
        inner.pout.store(pout + avaliable, Ordering::Release);
        Ok(avaliable)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(1), FIFO_MIN_CAPACITY);
        assert_eq!(align_up(3), FIFO_MIN_CAPACITY);
        assert_eq!(align_up(16), FIFO_MIN_CAPACITY);
        assert_eq!(align_up(255), 256);
    }
}
