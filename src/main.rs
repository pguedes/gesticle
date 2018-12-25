extern crate input;
extern crate libc;
extern crate libxdo;
extern crate nix;
extern crate udev;

use libxdo::XDo;

use events::listen;
use gestures::{PinchDirection, RotationDirection, SwipeDirection};
use gestures::GestureType::{Pinch, Rotation, Swipe};

mod gestures;
mod events;

fn main() {

    let xdo = XDo::new(None).expect("failed to create xdo ctx");

    listen(|t| {

        match t {
            Swipe(SwipeDirection::Up, 3) => xdo.send_keysequence("ctrl+t", 0).unwrap(),
            Swipe(SwipeDirection::Down, 3) => xdo.send_keysequence("ctrl+w", 0).unwrap(),
            Swipe(SwipeDirection::Left, 3) => xdo.send_keysequence("alt+Left", 0).unwrap(),
            Swipe(SwipeDirection::Right, 3) => xdo.send_keysequence("alt+Right", 0).unwrap(),

            Swipe(SwipeDirection::Up, 4) => xdo.send_keysequence("ctrl+alt+Up", 0).unwrap(),
            Swipe(SwipeDirection::Down, 4) => xdo.send_keysequence("ctrl+alt+Down", 0).unwrap(),
            Swipe(SwipeDirection::Left, 4) => xdo.send_keysequence("ctrl+Page_Up", 0).unwrap(),
            Swipe(SwipeDirection::Right, 4) => xdo.send_keysequence("ctrl+Page_Down", 0).unwrap(),

            Pinch(PinchDirection::Out, _) => xdo.send_keysequence("ctrl+plus", 0).unwrap(),
            Pinch(PinchDirection::In, _) => xdo.send_keysequence("ctrl+minus", 0).unwrap(),

            Rotation(RotationDirection::Left, _) => xdo.send_keysequence("ctrl+z", 0).unwrap(),
            Rotation(RotationDirection::Right, _) => xdo.send_keysequence("ctrl+shift+z", 0).unwrap(),

            _ => println!("{:?}", t)
        }
    });
}

