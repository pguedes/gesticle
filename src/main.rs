extern crate input;
extern crate libc;
extern crate nix;
extern crate udev;

use std::fmt;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use input::Event::Gesture;
use input::event::gesture::GestureEndEvent;
use input::event::gesture::GestureEventCoordinates;
use input::event::gesture::GestureEventTrait;
use input::event::gesture::GestureSwipeBeginEvent;
use input::event::gesture::GestureSwipeEndEvent;
use input::event::gesture::GestureSwipeEvent::Begin;
use input::event::gesture::GestureSwipeEvent::End;
use input::event::gesture::GestureSwipeEvent::Update;
use input::event::gesture::GestureSwipeUpdateEvent;
use input::event::GestureEvent::*;
use input::Libinput;
use input::LibinputInterface;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::close;
use udev::Context;
use std::f64::consts::PI;

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

struct DetectedGesture {
    dx: f64,
    dy: f64,
    fingers: i32
}

impl DetectedGesture {

    fn add (&mut self, dx: f64, dy: f64) {
        self.dx += dx;
        self.dy += dy;
    }

    fn direction(&self) -> String {
        let theta = (self.dy/self.dx).atan();
        let t: f64 = 180.into();
        let angle = theta * (t/PI);
        return angle.to_string();
    }
}

impl fmt::Debug for DetectedGesture {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}) fingers = {}", self.dx, self.dy, self.fingers)
    }
}

struct Listener {
    gesture: Option<DetectedGesture>
}

impl Listener {

    fn swipe_begin(&mut self, event: GestureSwipeBeginEvent) {

        let fingers    = event.finger_count();

        self.gesture = Some( DetectedGesture {
            dx: 0.0,
            dy: 0.0,
            fingers
        });

        println!("Swipe begin fingers = {:?}", fingers)
    }

    fn swipe_update(&mut self, event: GestureSwipeUpdateEvent) {

        let dx = event.dx();
        let dy = event.dy();

        match self.gesture {
            Some(ref mut g) => g.add(dx, dy),
            None => ()
        }

//        println!("Swipe update ({:?}, {:?})", dx, dy);
//        println!("Gesture {:?}", self.gesture);
    }

    fn swipe_end(self, event: GestureSwipeEndEvent) -> Result<DetectedGesture, Error> {

        let cancelled = event.cancelled();

        println!("Swipe end cancelled = {:?}", cancelled);

        self.gesture.expect("failed");
//        match self.gesture {
//            Some(ref g) => println!("Gesture {:?}, direction = {:?}", g, g.direction()),
//            None => ()
//        }

    }
}



fn main() {

    let mut listener = Listener { gesture: None };

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

