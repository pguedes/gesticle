use std::os::unix::io::RawFd;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use gestures::SwipeGesture;

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
use input::event::gesture::{GesturePinchEvent, GesturePinchBeginEvent, GesturePinchUpdateEvent, GesturePinchUpdateEvent};
use input::event::GestureEvent::*;
use input::Libinput;

use input::LibinputInterface;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::close;
use udev::Context;
use gestures::PinchGesture;

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

struct Listener {
    gesture: Option<SwipeGesture>
}

impl Listener {

    fn swipe_begin(&mut self, event: GestureSwipeBeginEvent) {

        let fingers    = event.finger_count();

        self.gesture = Some( SwipeGesture::new(fingers) );

        println!("Swipe begin fingers = {:?}", fingers)
    }

    fn swipe_update(&mut self, event: GestureSwipeUpdateEvent) {

        match self.gesture {
            Some(ref mut g) => g.add(event.dx(), event.dy()),
            None => ()
        }
    }

    fn swipe_end(self, event: GestureSwipeEndEvent) -> Result<SwipeGesture, String> {

        match self.gesture {
            Some(mut g) => {
                if event.cancelled() {
                    g.cancel();
                }
                Ok(g)
            },
            None => Err("failed to produce event".to_owned())
        }
    }

    fn pinch_begin(&mut self, event: GesturePinchBeginEvent) {

        let fingers    = event.finger_count();

        self.pinch = Some( SwipeGesture::new(fingers) );

        println!("Swipe begin fingers = {:?}", fingers)
    }

    fn pinch_update(&mut self, event: GesturePinchUpdateEvent) {

        match self.pinch {
            Some(ref mut g) => g.add(event.dx(), event.dy()),
            None => ()
        }
    }

    fn pinch_end(self, event: GesturePinchEndEvent) -> Result<PinchGesture, String> {

        match self.pinch {
            Some(mut g) => {
                if event.cancelled() {
                    g.cancel();
                }
                Ok(g)
            },
            None => Err("failed to produce event".to_owned())
        }
    }
}


pub fn listen<G, P>(swipe: G, pinch: P)
    where G: Fn(SwipeGesture), P: Fn(GesturePinchEvent) {

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
                Gesture(Swipe(End(event))) => {
                    match listener.swipe_end(event) {
                        Ok(g) => swipe(g),
                        Err(s) => println!("no Gesture {:?}", s)
                    }
                    listener = Listener { gesture: None }
                },

                Gesture(Pinch(GesturePinchEvent::Begin(event))) => listener.pinch_begin(event),
                Gesture(Pinch(GesturePinchEvent::Update(event))) => listener.pinch_update(event),
                Gesture(Pinch(GesturePinchEvent::End(event))) => {
                    match listener.pinch_end(event) {
                        Ok(p) => pinch(p),
                        Err(s) => println!("no Gesture {:?}", s)
                    }
                },

                Gesture(Pinch(event)) => pinch(event),
                _ => ()
            }

        }
        sleep(Duration::from_millis(10));
    }
}

