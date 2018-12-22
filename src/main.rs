extern crate input;
extern crate libc;
extern crate nix;
extern crate udev;

mod gestures;
mod events;

use events::listen;
use gestures::Identifiable;

fn main() {

    listen(|g| {

        match g.gesture_type() {
            Some(t) => println!("{:?}", t),
            _ => println!("unknown gesture: {:?}", g)
        }
    }, |p| {

        match p.gesture_type() {
            Some(t) => println!("{:?}", t),
            _ => println!("unknown gesture: {:?}", p)
        }
    });
}

