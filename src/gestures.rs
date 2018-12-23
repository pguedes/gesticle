use std::fmt;
use std::f64::consts::PI;

pub struct SwipeGesture {
    dx: f64,
    dy: f64,
    fingers: i32,
    cancelled: bool
}

impl SwipeGesture {

    pub fn new (fingers: i32) -> SwipeGesture {
        SwipeGesture { dx: 0.0, dy: 0.0, cancelled: false, fingers }
    }

    pub fn add (&mut self, dx: f64, dy: f64) {
        self.dx += dx;
        self.dy += dy;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn direction(&self) -> Option<SwipeDirection> {
        let theta = (self.dy/self.dx).atan();
        let t: f64 = 180.into();
        let angle = (theta * (t/PI)).abs();
        if 75.0 < angle && angle < 105.0 {
            return if self.dy > 0.0 {Some(SwipeDirection::Down)} else {Some(SwipeDirection::Up)}
        } else if 0.0 < angle && angle < 15.0 {
            return if self.dx > 0.0 {Some(SwipeDirection::Right)} else {Some(SwipeDirection::Left)}
        }
        println!("unknown direction: {:?} direction = {:?}", self, angle);
        return None;
    }
}

impl fmt::Debug for SwipeGesture {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}) fingers = {} cancelled? {}", self.dx, self.dy, self.fingers, self.cancelled)
    }
}

pub struct PinchGesture {
    scale: f64,
    dx: f64,
    dy: f64,
    angle: f64,
    cancelled: bool
}

impl PinchGesture {

    pub fn new (scale: f64) -> PinchGesture {
        PinchGesture { scale, dx: 0.0, dy: 0.0, angle: 0.0, cancelled: false }
    }

    pub fn add (&mut self, dx: f64, dy: f64, angle: f64) {
        self.dx += dx;
        self.dy += dy;
        self.angle += angle;
    }

    pub fn scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

}

impl fmt::Debug for PinchGesture {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "scale = {}, ({}, {}) angle = {} cancelled? {}",
               self.scale, self.dx, self.dy, self.angle, self.cancelled)
    }
}

#[derive(Debug)]
pub enum GestureType {
    Swipe(SwipeDirection, i32),
    Rotation(RotationDirection, f64),
    Pinch(PinchDirection, f64)
}
#[derive(Debug)]
pub enum SwipeDirection { Up, Down, Left, Right }
#[derive(Debug)]
pub enum RotationDirection { Left, Right }
#[derive(Debug)]
pub enum PinchDirection { In, Out }

pub trait Identifiable {

    fn gesture_type(&self) -> Option<GestureType>;
}

impl Identifiable for SwipeGesture {

    fn gesture_type(&self) -> Option<GestureType> {
        return match self.direction() {
            Some(d) => Some(GestureType::Swipe(d, self.fingers)),
            None => None
        };
    }
}

impl Identifiable for PinchGesture {

    fn gesture_type(&self) -> Option<GestureType> {
        if self.angle > 50.0 {
            return Some(GestureType::Rotation(RotationDirection::Right, self.angle));
        } else if self.angle < -50.0 {
            return Some(GestureType::Rotation(RotationDirection::Left, self.angle));
        }

        if self.scale > 1.0 {
            return Some(GestureType::Pinch(PinchDirection::Out, self.scale));
        } else if self.scale < 1.0 {
            return Some(GestureType::Pinch(PinchDirection::In, self.scale));
        }

        return None
    }
}