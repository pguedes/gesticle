use std::f64::consts::PI;
use std::fmt;

use input::Event::Gesture;
use input::event::gesture::{GesturePinchEndEvent, GesturePinchEvent, GesturePinchUpdateEvent};
use input::event::gesture::GestureEndEvent;
use input::event::gesture::GestureEventCoordinates;
use input::event::gesture::GestureEventTrait;
use input::event::gesture::GesturePinchEventTrait;
use input::event::gesture::GestureSwipeEndEvent;
use input::event::gesture::GestureSwipeEvent::Begin;
use input::event::gesture::GestureSwipeEvent::End;
use input::event::gesture::GestureSwipeEvent::Update;
use input::event::gesture::GestureSwipeUpdateEvent;
use input::event::GestureEvent::*;

struct SwipeGesture {
    dx: f64,
    dy: f64,
    fingers: i32,
    cancelled: bool
}

impl SwipeGesture {

    fn new (fingers: i32) -> SwipeGesture {
        SwipeGesture { dx: 0.0, dy: 0.0, cancelled: false, fingers }
    }

    fn add (&mut self, dx: f64, dy: f64) {
        self.dx += dx;
        self.dy += dy;
    }

    fn cancel(&mut self) {
        self.cancelled = true;
    }

    fn direction(&self) -> Option<SwipeDirection> {
        let theta = (self.dy/self.dx).atan();
        let t: f64 = 180.into();
        let angle = (theta * (t/PI)).abs();
        if 75.0 < angle && angle < 105.0 {
            return if self.dy > 0.0 {Some(SwipeDirection::Down)} else {Some(SwipeDirection::Up)}
        } else if 0.0 < angle && angle < 15.0 {
            return if self.dx > 0.0 {Some(SwipeDirection::Right)} else {Some(SwipeDirection::Left)}
        }
        warn!("unknown direction: {:?} direction = {:?}", self, angle);
        return None;
    }
}

impl fmt::Debug for SwipeGesture {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}) fingers = {} cancelled? {}", self.dx, self.dy, self.fingers, self.cancelled)
    }
}

struct PinchGesture {
    scale: f64,
    dx: f64,
    dy: f64,
    angle: f64,
    cancelled: bool
}

impl PinchGesture {

    fn new (scale: f64) -> PinchGesture {
        PinchGesture { scale, dx: 0.0, dy: 0.0, angle: 0.0, cancelled: false }
    }

    fn add (&mut self, dx: f64, dy: f64, angle: f64) {
        self.dx += dx;
        self.dy += dy;
        self.angle += angle;
    }

    fn scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    fn cancel(&mut self) {
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

impl GestureType {
    pub fn to_config(&self) -> String {
        match self {
            GestureType::Swipe(direction, fingers) => format!("swipe.{:?}.{}", direction, fingers),
            GestureType::Rotation(direction, _) => format!("rotation.{:?}", direction),
            GestureType::Pinch(direction, _) => format!("pinch.{:?}", direction)
        }
    }
}

#[derive(Debug)]
pub enum SwipeDirection { Up, Down, Left, Right }
#[derive(Debug)]
pub enum RotationDirection { Left, Right }
#[derive(Debug)]
pub enum PinchDirection { In, Out }

trait Identifiable {

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

struct SwipeBuilder {
    swipe: Option<SwipeGesture>
}

impl SwipeBuilder {

    fn empty() -> SwipeBuilder {
        SwipeBuilder { swipe: None }
    }

    fn new(&mut self, fingers: i32) {
        self.swipe = Some( SwipeGesture::new(fingers) );
    }

    fn update(&mut self, event: GestureSwipeUpdateEvent) {

        match self.swipe {
            Some(ref mut g) => g.add(event.dx(), event.dy()),
            None => ()
        }
    }

    fn build(self, event: GestureSwipeEndEvent) -> Result<SwipeGesture, String> {

        match self.swipe {
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

struct PinchBuilder {
    pinch: Option<PinchGesture>
}

impl PinchBuilder {

    fn empty() -> PinchBuilder {
        PinchBuilder {
            pinch: None
        }
    }

    fn new(&mut self, scale: f64) {
        self.pinch = Some( PinchGesture::new(scale) );
    }

    fn update(&mut self, event: GesturePinchUpdateEvent) {

        match self.pinch {
            Some(ref mut g) => g.add(event.dx(), event.dy(), event.angle_delta()),
            None => ()
        }
    }

    fn build(self, event: GesturePinchEndEvent) -> Result<PinchGesture, String> {

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


pub struct Listener<'a> {
    swipe: SwipeBuilder,
    pinch: PinchBuilder,
    gesture_action: &'a dyn Fn(GestureType)
}

impl<'a> Listener<'a> {

    pub fn new(gesture_action: &dyn Fn(GestureType)) -> Listener {
        Listener {
            swipe: SwipeBuilder::empty(),
            pinch: PinchBuilder::empty(),
            gesture_action
        }
    }

    pub fn event(mut self, event: input::Event) -> Self {

        match event {
            Gesture(Swipe(Begin(event))) =>
                self.swipe.new(event.finger_count()),
            Gesture(Swipe(Update(event))) =>
                self.swipe.update(event),
            Gesture(Swipe(End(event))) => {
                match self.swipe.build(event) {
                    Ok(g) => {
                        match g.gesture_type() {
                            Some(t) => (self.gesture_action)(t),
                            None => error!("unrecognized gesture {:?}", g)
                        }
                    },
                    Err(s) => error!("no Gesture {:?}", s)
                }
                self.swipe = SwipeBuilder::empty();
            },

            Gesture(Pinch(GesturePinchEvent::Begin(event))) =>
                self.pinch.new(event.scale()),
            Gesture(Pinch(GesturePinchEvent::Update(event))) =>
                self.pinch.update(event),
            Gesture(Pinch(GesturePinchEvent::End(event))) => {
                let gesture = self.pinch.build(event);
                match gesture {
                    Ok(p) => {
                        match p.gesture_type() {
                            Some(t) => (self.gesture_action)(t),
                            None => error!("unrecognized gesture {:?}", p)
                        }
                    },
                    Err(s) => error!("no Gesture {:?}", s)
                }
                self.pinch = PinchBuilder::empty();
            },

            _ => ()
        }
        self
    }
}