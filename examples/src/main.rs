extern crate fifo;
use std::{fs, thread, time};
use std::io::prelude::*;
use fifo::fifo;

fn main() {
    let (mut sender, mut receiver) = fifo(1 << 20).unwrap();
    let write_thread = thread::Builder::new().name("write".to_owned())
        .spawn(move || {
        let mut f = fs::File::open("./rust.avi").unwrap();
        let mut buffer = [0 as u8; 8192];
        loop {
            let bytes = f.read(&mut buffer).unwrap();
            sender.write_all(&buffer[0..bytes]).unwrap();
            if bytes < buffer.len() {
                break;
            }
        }
    }).unwrap();
    let read_thread = thread::Builder::new().name("read".to_owned()).
        spawn(move || {
        let mut f = fs::OpenOptions::new()
            .write(true).create(true).open("./tsur.avi").unwrap();
        let mut buffer = [0 as u8; 8192];
        loop {
            let bytes = receiver.read(&mut buffer).unwrap();
            f.write_all(&buffer[0..bytes]).unwrap();
            if bytes == 0 {
                break;
            }
        }
    }).unwrap();
    write_thread.join().unwrap();
    read_thread.join().unwrap();
    println!("all threads are exited, all resources should be released");
    thread::sleep(time::Duration::from_secs(60));
}
