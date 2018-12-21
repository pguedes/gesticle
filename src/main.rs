extern crate input;
extern crate libc;
extern crate nix;
extern crate udev;

mod gestures;
mod events;

use events::listen;

fn main() {

    listen(|g| {
        println!("Gesture {:?} with direction {:?}", g, g.direction())
    }, |p| println!("Pinch {:?}", p));
}

