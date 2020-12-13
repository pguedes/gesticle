use std::os::unix::io::RawFd;
use std::path::Path;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::close;
use udev::Context;

use input::Libinput;
use input::LibinputInterface;

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

pub trait EventSink {
    fn event(&mut self, event: input::Event);
}

impl<F> EventSink for F where F: FnMut(input::Event) {
    fn event(&mut self, event: input::Event) {
        self(event);
    }
}

pub struct EventSource;

impl EventSource {

    pub fn listen<S>(sink: &mut S) where S: EventSink {

        let io = LibInputFile { };
        let ctx = Context::new().expect("could not create udev context...");
        let mut libinput = Libinput::new_from_udev(io, &ctx);

        libinput.udev_assign_seat("seat0").unwrap();

        loop {
            libinput.dispatch().unwrap();
            while let Some(event) = libinput.next() {
                sink.event(event);
            }
        }
    }
}
