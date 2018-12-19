extern crate input;
extern crate libc;
extern crate nix;
extern crate udev;

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use input::Libinput;
use input::LibinputInterface;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::close;
use udev::Context;

struct LibInputFile;

impl LibinputInterface for LibInputFile {

    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        if let Ok(fd) = open(path, OFlag::from_bits_truncate(flags), Mode::empty()) {
            Ok(fd)
        } else {
            Err(1)
        }
    }

    fn close_restricted(&mut self, fd: RawFd) {
        let _ = close(fd);
    }
}



fn main() {

    let io = LibInputFile { };
    let ctx = Context::new().expect("could not create udev context...");
    let mut libinput = Libinput::new_from_udev(io, &ctx);

    libinput.udev_assign_seat("seat0").unwrap();

    loop {
        libinput.dispatch().unwrap();
        while let Some(event) = libinput.next() {
            println!("that was ok: {:?}", event);
        }
        sleep(Duration::from_millis(10));
    }
}

