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

pub struct GestureSource {
    pinch_in_scale_trigger: f64,
    pinch_out_scale_trigger: f64,
}

impl GestureSource {

    pub fn new(pinch_in_scale_trigger: f64, pinch_out_scale_trigger: f64) -> GestureSource {
        GestureSource {
            pinch_in_scale_trigger,
            pinch_out_scale_trigger
        }
    }

    pub fn listen(&self) -> mpsc::Receiver<GestureType> {
        let (tx, rx) = mpsc::channel();

        let pinch_in_trigger = self.pinch_in_scale_trigger;
        let pinch_out_trigger = self.pinch_out_scale_trigger;

        thread::spawn(move || {

            let io = LibInputFile { };
            let ctx = Context::new().expect("could not create udev context...");
            let mut libinput = Libinput::new_from_udev(io, &ctx);

            libinput.udev_assign_seat("seat0").unwrap();
            let publish = |t: GestureType| tx.send(t).unwrap();
            let mut listener =
                Listener::new(pinch_in_trigger, pinch_out_trigger, &publish);

            loop {
                libinput.dispatch().unwrap();
                while let Some(event) = libinput.next() {
                    listener.event(event);
                }
                sleep(Duration::from_millis(10));
            }
        });

        return rx
    }
}
