use std::os::unix::io::RawFd;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use input::Event::Gesture;
use input::event::gesture::{GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchEvent, GesturePinchUpdateEvent};
use input::event::gesture::GestureEndEvent;
use input::event::gesture::GestureEventCoordinates;
use input::event::gesture::GestureEventTrait;
use input::event::gesture::GesturePinchEventTrait;
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

use gestures::PinchGesture;
use gestures::SwipeGesture;

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
    gesture: Option<SwipeGesture>,
    pinch: Option<PinchGesture>
}

impl Listener {

    fn swipe_begin(&mut self, event: GestureSwipeBeginEvent) {
        self.gesture = Some( SwipeGesture::new(event.finger_count()) );
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
        self.pinch = Some( PinchGesture::new(event.scale()) );
    }

    fn pinch_update(&mut self, event: GesturePinchUpdateEvent) {

        match self.pinch {
            Some(ref mut g) => g.add(event.dx(), event.dy(), event.angle_delta()),
            None => ()
        }
    }

    fn pinch_end(self, event: GesturePinchEndEvent) -> Result<PinchGesture, String> {

        match self.pinch {
            Some(mut g) => {
                g.scale(event.scale());
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
    where G: Fn(SwipeGesture), P: Fn(PinchGesture) {

    let mut listener = Listener { gesture: None, pinch: None };

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
                    listener = Listener { gesture: None, pinch: None }
                },

                Gesture(Pinch(GesturePinchEvent::Begin(event))) => listener.pinch_begin(event),
                Gesture(Pinch(GesturePinchEvent::Update(event))) => listener.pinch_update(event),
                Gesture(Pinch(GesturePinchEvent::End(event))) => {
                    match listener.pinch_end(event) {
                        Ok(p) => pinch(p),
                        Err(s) => println!("no Gesture {:?}", s)
                    }
                    listener = Listener { gesture: None, pinch: None }
                },

                _ => ()
            }

        }
        sleep(Duration::from_millis(10));
    }
}

