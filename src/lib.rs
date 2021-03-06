//! A first-in-first-out bytes ring-buffer like kfifo in Linux.
//!
//! # Example
//!
//! ```
//! use std::io::prelude::*;
//! use fifo::{fifo, Sender, Receiver};
//!
//! let (mut sender, mut receiver): (Sender, Receiver) = fifo(128);
//!
//! let bytes_to_write = [0 as u8; 512];
//! assert_eq!(sender.write(&bytes_to_write).unwrap(), 128);
//!
//! let mut bytes_to_read = [1 as u8; 512];
//! assert_eq!(receiver.read(&mut bytes_to_read).unwrap(), 128);
//!
//! assert_eq!(bytes_to_write[0..128], bytes_to_read[0..128]);
//!
//! assert!(sender.as_ref().unread() >= 0);
//! ```
#![feature(alloc, heap_api)]
#![feature(optin_builtin_traits)]
extern crate alloc;
mod shutdown;
pub mod splice;
use std::{io, slice, thread, time};
use std::cmp::min;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use alloc::heap;
use shutdown::{Shutdown, SHUT_READ, SHUT_WRITE};

struct Inner {
    buffer: *mut u8,
    size: usize,
    pin: AtomicUsize,
    pout: AtomicUsize,
    shutdown: Shutdown,
}

impl Inner {
    fn new(size: usize) -> Self {
        let shutdown = Shutdown::new();
        Inner {
            buffer: unsafe { heap::allocate(size, 1) },
            size: size,
            pin: AtomicUsize::new(0),
            pout: AtomicUsize::new(0),
            shutdown: shutdown,
        }
    }
    pub fn unread(&self) -> usize {
        let pout = self.pout.load(Ordering::Acquire);
        let pin = self.pin.load(Ordering::Acquire);
        pin - pout
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe { heap::deallocate(self.buffer, self.size, 1); }
    }
}

unsafe impl Sync for Inner {}

/// What can we do when operations on `Sender` or `Receiver` would block.
///
/// 1) Just return `Err(ErrorKind::WouldBlock)` immediately; or
/// 2) sleep for some milliseconds.
///
/// The default on `Sender` and `Receiver` is Sleep(10).
pub enum WouldBlock {
    Nonblock,
    Sleep(u64),
}

/// The fifo sender. It's `Send` but `!Sync`.
pub struct Sender {
    _private: (),
    inner: Arc<Inner>,
    would_block: WouldBlock,
}

/// The fifo receiver. It's `Send` but `!Sync`.
pub struct Receiver {
    _private: (),
    inner: Arc<Inner>,
    would_block: WouldBlock,
}

impl AsRef<Inner> for Sender {
    fn as_ref(&self) -> &Inner {
        &self.inner
    }
}

impl AsRef<Inner> for Receiver {
    fn as_ref(&self) -> &Inner {
        &self.inner
    }
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

/// Construct the fifo with capacity as `size.next_power_of_two()`,
/// and return the `Sender` and `Receiver` pair connected with that.
pub fn fifo(size: usize) -> (Sender, Receiver) {
    let size = size.next_power_of_two();
    let inner = Arc::new(Inner::new(size));
    let sender = Sender {
        _private: (),
        inner: inner.clone(),
        would_block: WouldBlock::Sleep(10),
    };
    let receiver = Receiver {
        _private: (),
        inner: inner,
        would_block: WouldBlock::Sleep(10),
    };
    (sender, receiver)
}

impl Sender {
    pub fn set_would_block(&mut self, would_block: WouldBlock) {
        self.would_block = would_block;
    }

    fn do_write<T>(&mut self, bytes: usize, mut cp_data_to: T) -> io::Result<usize>
        where T: FnMut(&mut [u8], usize, usize) -> io::Result<usize>
    {
        let inner: &Inner = &self.inner;
        let mut pin: usize;
        let mut pout: usize;
        let mut avaliable: usize;
        loop {
            pin = inner.pin.load(Ordering::Relaxed);
            pout = inner.pout.load(Ordering::Acquire);
            avaliable = min(inner.size - (pin - pout), bytes);
            if avaliable > 0 {
                break;
            } else {
                if inner.shutdown.shuted(SHUT_READ) {
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed on read end"));
                }
                if let WouldBlock::Sleep(sleep) = self.would_block {
                    thread::sleep(time::Duration::from_millis(sleep));
                } else {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock, "buffer is full"));
                }
            };
        }
        let start_pos = pin & (inner.size - 1);
        let mut dest = unsafe { slice::from_raw_parts_mut(inner.buffer, inner.size) };

        let exactly_write = cp_data_to(&mut dest, start_pos, avaliable)?;
        inner.pin.store(pin + exactly_write, Ordering::Release);
        Ok(exactly_write)
    }
}

impl io::Write for Sender {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let bytes_len = bytes.len();
        let copy_data_to = move |dest: &mut [u8], start_pos: usize, avaliable: usize| {
            let len_to_write_1 = min(dest.len() - start_pos, avaliable);
            let len_to_write_2 = avaliable - len_to_write_1;
            dest[start_pos..(start_pos+len_to_write_1)].copy_from_slice(&bytes[0..len_to_write_1]);
            dest[0..len_to_write_2].copy_from_slice(&bytes[len_to_write_1..avaliable]);
            Ok(avaliable)
        };
        self.do_write(bytes_len, copy_data_to)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Receiver {
    pub fn set_would_block(&mut self, would_block: WouldBlock) {
        self.would_block = would_block;
    }

    fn do_write<T>(&mut self, bytes: usize, mut cp_data_from: T) -> io::Result<usize>
        where T: FnMut(&[u8], usize, usize) -> io::Result<usize>
    {
        let inner: &Inner = &self.inner;
        let mut pin: usize;
        let mut pout: usize;
        let mut avaliable: usize;
        loop {
            pin = inner.pin.load(Ordering::Acquire);
            pout = inner.pout.load(Ordering::Relaxed);
            avaliable = min(bytes, pin - pout);
            if avaliable > 0 {
                break;
            } else {
                if inner.shutdown.shuted(SHUT_WRITE) {
                    return Ok(0);
                }
                if let WouldBlock::Sleep(sleep) = self.would_block {
                    thread::sleep(time::Duration::from_millis(sleep));
                } else {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock, "buffer is empty"));
                }
            }
        }
        let start_pos = pout & (inner.size - 1);
        let src = unsafe { slice::from_raw_parts_mut(inner.buffer, inner.size) };
        let exactly_read = cp_data_from(&src, start_pos, avaliable)?;
        inner.pout.store(pout + exactly_read, Ordering::Release);
        Ok(exactly_read)
    }
}

impl io::Read for Receiver {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let bytes_len = bytes.len();
        let copy_data_from = move |src: &[u8], start_pos: usize, avaliable: usize| {
            let len_to_read_1 = min(src.len() - start_pos, avaliable);
            let len_to_read_2 = avaliable - len_to_read_1;
            bytes[0..len_to_read_1].copy_from_slice(&src[start_pos..(start_pos+len_to_read_1)]); 
            bytes[len_to_read_1..avaliable].copy_from_slice(&src[0..len_to_read_2]);
            Ok(avaliable)
        };
        self.do_write(bytes_len, copy_data_from)
    }
}
