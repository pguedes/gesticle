use std::os::unix::io::RawFd;
use std::path::Path;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::close;
use udev::Context;

use input::Libinput;
use input::LibinputInterface;

use gestures::GestureType;
use gestures::Listener;

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


pub fn listen<G>(gesture_action: G)
    where G: Fn(GestureType) {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {

        let io = LibInputFile { };
        let ctx = Context::new().expect("could not create udev context...");
        let mut libinput = Libinput::new_from_udev(io, &ctx);

        libinput.udev_assign_seat("seat0").unwrap();
        let publish = |t: GestureType| tx.send(t).unwrap();
        let mut listener = Listener::new(&publish);

        loop {
            libinput.dispatch().unwrap();
            while let Some(event) = libinput.next() {
                listener = listener.event(event);
            }
            sleep(Duration::from_millis(10));
        }
    });

    for event in rx {
        gesture_action(event);
    }
}
