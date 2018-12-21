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

    pub fn direction(&self) -> String {
        let theta = (self.dy/self.dx).atan();
        let t: f64 = 180.into();
        let angle = theta * (t/PI);
        return angle.to_string();
    }
}

impl fmt::Debug for SwipeGesture {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}) fingers = {} cancelled? {}", self.dx, self.dy, self.fingers, self.cancelled)
    }
}

pub struct PinchGesture {
    dx: f64,
    dy: f64,
    fingers: i32,
    cancelled: bool
}

impl PinchGesture {

    pub fn new (fingers: i32) -> PinchGesture {
        PinchGesture { dx: 0.0, dy: 0.0, cancelled: false, fingers }
    }

    pub fn add (&mut self, dx: f64, dy: f64) {
        self.dx += dx;
        self.dy += dy;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn direction(&self) -> String {
        let theta = (self.dy/self.dx).atan();
        let t: f64 = 180.into();
        let angle = theta * (t/PI);
        return angle.to_string();
    }
}

impl fmt::Debug for PinchGesture {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}) fingers = {} cancelled? {}", self.dx, self.dy, self.fingers, self.cancelled)
    }
}
