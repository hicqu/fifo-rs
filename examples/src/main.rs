extern crate fifo;
use std::{fs, thread};
use std::io::prelude::*;
use fifo::{fifo, Sender, Receiver};
use fifo::splice::{SpliceRead, SpliceWrite};

fn get_write_thread(mut sender: Sender, file: &'static str) -> thread::JoinHandle<()> {
    thread::Builder::new().name("write".to_owned()).spawn(move || {
        let mut f = fs::File::open(file).unwrap();
        let mut buffer = [0 as u8; 8192];
        loop {
            let bytes = f.read(&mut buffer).unwrap();
            sender.write_all(&buffer[0..bytes]).unwrap();
            if bytes < buffer.len() {
                break;
            }
        }
    }).unwrap()
}

fn get_read_thread(mut receiver: Receiver, file: &'static str) -> thread::JoinHandle<()> {
    thread::Builder::new().name("read".to_owned()).spawn(move || {
        let mut f = fs::OpenOptions::new()
            .write(true).create(true).open(file).unwrap();
        let mut buffer = [0 as u8; 8192];
        loop {
            let bytes = receiver.read(&mut buffer).unwrap();
            f.write_all(&buffer[0..bytes]).unwrap();
            if bytes == 0 {
                break;
            }
        }
    }).unwrap()
}

fn get_write_splice_thread(mut sender: Sender, file: &'static str) -> thread::JoinHandle<()> {
    thread::Builder::new().name("write splice".to_owned()).spawn(move || {
        let mut f = fs::File::open(file).unwrap();
        sender.splice_all_from(&mut f).unwrap();
    }).unwrap()
}

fn get_read_splice_thread(mut receiver: Receiver, file: &'static str) -> thread::JoinHandle<()> {
    thread::Builder::new().name("read splice".to_owned()).spawn(move || {
        let mut f = fs::OpenOptions::new()
            .write(true).create(true).open(file).unwrap();
        receiver.splice_all_to(&mut f).unwrap();
    }).unwrap()
}

fn main() {
    let (sender, receiver) = fifo(1 << 20);
    let write_thread = get_write_thread(sender, "rust.avi");
    let read_thread = get_read_thread(receiver, "tsur.avi");
    write_thread.join().unwrap();
    read_thread.join().unwrap();
    println!("all threads are exited");

    let (sender, receiver) = fifo(1 << 20);
    let write_thread = get_write_splice_thread(sender, "rust1.avi");
    let read_thread = get_read_splice_thread(receiver, "tsur1.avi");
    write_thread.join().unwrap();
    read_thread.join().unwrap();
    println!("all threads are exited");
}
