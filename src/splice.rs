//! Zero-copy interface for `Sender` and `Receiver`, in **splice** style in Linux.
//! When you want to write bytes into the ring-buffer from a `Read`, or read
//! bytes from this into a `Write`, you can use `SpliceRead` and `SpliceWrite`.
use std::io;
use std::cmp::min;
use super::{Sender, Receiver};

/// Copy some bytes directly from another `Read` object, without use any temporary buffers.
pub trait SpliceRead {
    /// Copy at most *bytes* bytes from a `Read` *r*.
    fn splice_from<T>(&mut self, r: &mut T, bytes: usize) -> io::Result<usize> where T: io::Read;

    /// Copy all bytes from a `Read` *r*.
    fn splice_all_from<T>(&mut self, r: &mut T) -> io::Result<()> where T: io::Read {
        loop {
            let bytes = self.splice_from(r, ::std::usize::MAX)?;
            if bytes == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl SpliceRead for Sender {
    fn splice_from<T>(&mut self, r: &mut T, bytes: usize) -> io::Result<usize> where T: io::Read {
        let cp_data_to = |dest: &mut [u8], start_pos: usize, avaliable: usize| {
            assert!(avaliable != 0);
            let len_to_write_1 = min(dest.len() - start_pos, avaliable);
            let len_to_write_2 = avaliable - len_to_write_1;

            let mut total_bytes = r.read(&mut dest[start_pos..(start_pos+len_to_write_1)])?;
            if total_bytes == len_to_write_1 {
                total_bytes += r.read(&mut dest[0..len_to_write_2])?;
            }
            Ok(total_bytes)
        };
        self.do_write(bytes, cp_data_to)
    }
}

/// Copy some bytes directly into another `Write` object, without use any temporary buffers.
pub trait SpliceWrite {
    /// Copy at most *bytes* bytes into a `Write` *w*.
    fn splice_to<T>(&mut self, w: &mut T, bytes: usize) -> io::Result<usize> where T: io::Write;

    /// Copy all bytes to a `Write` *w*.
    fn splice_all_to<T>(&mut self, w: &mut T) -> io::Result<()> where T: io::Write {
        loop {
            let bytes = self.splice_to(w, ::std::usize::MAX)?;
            if bytes == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl SpliceWrite for Receiver {
    fn splice_to<T>(&mut self, w: &mut T, bytes: usize) -> io::Result<usize> where T: io::Write {
        let cp_data_from = |src: &[u8], start_pos: usize, avaliable: usize| {
            assert!(avaliable != 0);
            let len_to_read_1 = min(src.len() - start_pos, avaliable);
            let len_to_read_2 = avaliable - len_to_read_1;

            let mut total_bytes = w.write(&src[start_pos..(start_pos+len_to_read_1)])?;
            if total_bytes == len_to_read_1 {
                total_bytes += w.write(&src[0..len_to_read_2])?;
            }
            Ok(total_bytes)
        };
        self.do_write(bytes, cp_data_from)
    }
}
