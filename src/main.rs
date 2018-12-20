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
use input::Event::Gesture;
use input::event::GestureEvent::*;

use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::close;
use udev::Context;
use input::event::gesture::GestureSwipeBeginEvent;
use input::event::gesture::GestureSwipeUpdateEvent;
use input::event::gesture::GestureSwipeEndEvent;
use input::event::gesture::GestureSwipeEvent;
use input::event::gesture::GestureSwipeEvent::Begin;
use input::event::gesture::GestureSwipeEvent::Update;
use input::event::gesture::GestureSwipeEvent::End;
use input::event::gesture::GestureEventTrait;
use input::event::gesture::GestureEventCoordinates;
use input::event::gesture::GestureEndEvent;

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

struct Listener;

impl Listener {

    fn swipe_begin(&mut self, event: GestureSwipeBeginEvent) {

        let fingers    = event.finger_count();

        println!("Swipe begin fingers = {:?}", fingers)
    }

    fn swipe_update(&mut self, event: GestureSwipeUpdateEvent) {

        let dx = event.dx();
        let dy = event.dy();

        println!("Swipe update ({:?}, {:?})", dx, dy)
    }

    fn swipe_end(&mut self, event: GestureSwipeEndEvent) {

        let cancelled = event.cancelled();

        println!("Swipe end cancelled = {:?}", cancelled)
    }
}



fn main() {

    let mut listener = Listener { };

    let io = LibInputFile { };
    let ctx = Context::new().expect("could not create udev context...");
    let mut libinput = Libinput::new_from_udev(io, &ctx);

    libinput.udev_assign_seat("seat0").unwrap();

    loop {
        libinput.dispatch().unwrap();
        while let Some(event) = libinput.next() {

            match event {
                Gesture(Swipe(Begin(event))) => listener.swipe_begin(event),
                Gesture(Swipe(Update(event))) => listener.swipe_update(event),
                Gesture(Swipe(End(event))) => listener.swipe_end(event),
                Gesture(Pinch(event)) => println!("Pinch {:?}", event),
                _ => ()
            }

        }
        sleep(Duration::from_millis(10));
    }
}

