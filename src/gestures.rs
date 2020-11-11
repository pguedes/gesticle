use std::f64::consts::PI;
use std::fmt;

use input::event::gesture::GestureEndEvent;
use input::event::gesture::GestureEventCoordinates;
use input::event::gesture::GestureEventTrait;
use input::event::gesture::GesturePinchEventTrait;
use input::event::gesture::GestureSwipeEndEvent;
use input::event::gesture::GestureSwipeEvent::Begin;
use input::event::gesture::GestureSwipeEvent::End;
use input::event::gesture::GestureSwipeEvent::Update;
use input::event::gesture::GestureSwipeUpdateEvent;
use input::event::gesture::{GesturePinchEndEvent, GesturePinchEvent, GesturePinchUpdateEvent};
use input::event::GestureEvent::*;
use input::Event::Gesture;
use std::fmt::Formatter;
use std::mem::swap;

#[derive(Copy, Clone)]
struct SwipeGesture {
    dx: f64,
    dy: f64,
    fingers: i32,
    cancelled: bool,
}

impl SwipeGesture {
    fn new(fingers: i32) -> SwipeGesture {
        SwipeGesture {
            dx: 0.0,
            dy: 0.0,
            cancelled: false,
            fingers,
        }
    }

    fn add(&mut self, dx: f64, dy: f64) {
        self.dx += dx;
        self.dy += dy;
    }

    fn cancel(&mut self) {
        self.cancelled = true;
    }

    fn direction(&self) -> Option<SwipeDirection> {
        let theta = (self.dy / self.dx).atan();
        let t: f64 = 180.into();
        let angle = (theta * (t / PI)).abs();
        if 75.0 < angle && angle < 105.0 {
            return if self.dy > 0.0 {
                Some(SwipeDirection::Down)
            } else {
                Some(SwipeDirection::Up)
            };
        } else if 0.0 < angle && angle < 15.0 {
            return if self.dx > 0.0 {
                Some(SwipeDirection::Right)
            } else {
                Some(SwipeDirection::Left)
            };
        }
        warn!("unknown direction: {:?} direction = {:?}", self, angle);
        return None;
    }
}

impl fmt::Debug for SwipeGesture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}, {}) fingers = {} cancelled? {}",
            self.dx, self.dy, self.fingers, self.cancelled
        )
    }
}

#[derive(Copy, Clone)]
struct PinchGesture {
    initial_scale: f64,
    scale: f64,
    dx: f64,
    dy: f64,
    angle: f64,
    cancelled: bool,
}

impl PinchGesture {
    fn new(scale: f64) -> PinchGesture {
        PinchGesture {
            initial_scale: scale,
            scale: 0.0,
            dx: 0.0,
            dy: 0.0,
            angle: 0.0,
            cancelled: false,
        }
    }

    fn add(&mut self, dx: f64, dy: f64, angle: f64, scale: f64) {
        self.dx += dx;
        self.dy += dy;
        self.angle += angle;
        self.scale = self.initial_scale - scale;
    }

    fn is_rotation(&self) -> bool {
        self.angle > 50.0 || self.angle < -50.0
    }

    fn rotation_direction(&self) -> Option<RotationDirection> {
        if self.is_rotation() {
            return RotationDirection::of_angle(self.angle);
        }
        None
    }

    fn direction(&self) -> Option<PinchDirection> {
        if !self.is_rotation() {
            return PinchDirection::of_scale(self.scale);
        }
        None
    }

    fn cancel(&mut self) {
        self.cancelled = true;
    }
}

impl fmt::Debug for PinchGesture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "scale = {}, ({}, {}) angle = {} cancelled? {}",
            self.scale, self.dx, self.dy, self.angle, self.cancelled
        )
    }
}

#[derive(Debug)]
pub enum GestureType {
    Swipe(SwipeDirection, i32),
    Rotation(RotationDirection, f64),
    Pinch(PinchDirection, f64),
}

impl GestureType {
    pub fn to_config(&self) -> String {
        match self {
            GestureType::Swipe(direction, fingers) => format!("swipe.{}.{}", direction, fingers),
            GestureType::Rotation(direction, _) => format!("rotation.{}", direction),
            GestureType::Pinch(direction, _) => format!("pinch.{}", direction),
        }
    }
}

#[derive(Debug)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum RotationDirection {
    Left,
    Right,
}

impl RotationDirection {
    fn of_angle(angle: f64) -> Option<RotationDirection> {
        if angle > 50.0 {
            return Some(RotationDirection::Right)
        } else if angle < -50.0 {
            return Some(RotationDirection::Left)
        }
        None
    }
}

#[derive(Debug)]
pub enum PinchDirection {
    In,
    Out,
}

impl PinchDirection {
    fn of_scale(scale: f64) -> Option<PinchDirection> {
        if scale > 0.0 {
            return Some(PinchDirection::In)
        } else if scale < 0.0 {
            return Some(PinchDirection::Out)
        }
        None
    }
}

impl fmt::Display for SwipeDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl fmt::Display for RotationDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl fmt::Display for PinchDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

trait Identifiable {
    fn gesture_type(&self) -> Option<GestureType>;
}

impl Identifiable for SwipeGesture {
    fn gesture_type(&self) -> Option<GestureType> {
        if self.cancelled {
            return None;
        }

        return match self.direction() {
            Some(d) => Some(GestureType::Swipe(d, self.fingers)),
            None => None,
        };
    }
}

impl Identifiable for PinchGesture {
    fn gesture_type(&self) -> Option<GestureType> {
        if !self.cancelled {
            return match self.rotation_direction() {
                Some(d) => Some(GestureType::Rotation(d, self.angle)),
                None => match self.direction() {
                    Some(d) => Some(GestureType::Pinch(d, self.scale)),
                    None => None
                }
            }
        }
        return None;
    }
}

struct SwipeBuilder {
    swipe: Option<SwipeGesture>,
}

impl SwipeBuilder {
    fn empty() -> SwipeBuilder {
        SwipeBuilder { swipe: None }
    }

    fn new(&mut self, fingers: i32) {
        self.swipe = Some(SwipeGesture::new(fingers));
    }

    fn update(&mut self, event: GestureSwipeUpdateEvent) {
        match self.swipe {
            Some(ref mut g) => g.add(event.dx(), event.dy()),
            None => (),
        }
    }

    fn build(&mut self, event: GestureSwipeEndEvent) -> Result<SwipeGesture, String> {

        // here we dont use copy semantics we simply consume the gesture and reset state on builder
        let mut swipe: Option<SwipeGesture> = None;
        swap(&mut swipe, &mut self.swipe);

        match swipe {
            Some(mut g) => {
                if event.cancelled() {
                    g.cancel();
                }
                Ok(g)
            }
            None => Err("failed to produce event".to_owned()),
        }
    }
}

struct PinchBuilder {
    pinch: Option<PinchGesture>,
    pinch_in_scale_trigger: f64,
    pinch_out_scale_trigger: f64
}

impl PinchBuilder {
    fn empty() -> PinchBuilder {
        PinchBuilder {
            pinch: None,
            pinch_in_scale_trigger: 0.0,
            pinch_out_scale_trigger: 0.0,
        }
    }

    fn new(&mut self, scale: f64, pinch_in_scale_trigger: f64, pinch_out_scale_trigger: f64) {
        self.pinch = Some(PinchGesture::new(scale));
        self.pinch_in_scale_trigger = pinch_in_scale_trigger;
        self.pinch_out_scale_trigger = pinch_out_scale_trigger;
    }

    fn update(&mut self, event: &GesturePinchUpdateEvent) -> Option<PinchGesture> {
        match self.pinch {
            Some(mut g) => {
                g.add(event.dx(), event.dy(), event.angle_delta(), event.scale());

                match g.direction() {
                    Some(PinchDirection::In) => {
                        // debug!("pinch in variation: {}", g.scale);
                        if self.pinch_in_scale_trigger != 0.0 && g.scale >= self.pinch_in_scale_trigger {
                            return Some(g)
                        }
                    },
                    Some(PinchDirection::Out) => {
                        // debug!("pinch out variation: {}", g.scale);
                        if self.pinch_out_scale_trigger != 0.0 && g.scale <= self.pinch_out_scale_trigger {
                            return Some(g)
                        }
                    },
                    None => return None
                }
                None
            }
            None => None
        }
    }

    fn build(&mut self, event: GesturePinchEndEvent) -> Result<PinchGesture, String> {

        // here we dont use copy semantics we simply consume the gesture and reset state on builder
        let mut pinch: Option<PinchGesture> = None;
        swap(&mut pinch, &mut self.pinch);

        match pinch {
            Some(mut g) => {
                if event.cancelled() {
                    g.cancel();
                }
                Ok(g)
            }
            None => Err("failed to produce event".to_owned()),
        }
    }
}

pub struct Listener<'a> {
    swipe: SwipeBuilder,
    pinch: PinchBuilder,
    gesture_action: &'a dyn Fn(GestureType),
    pinch_in_scale_trigger: f64,
    pinch_out_scale_trigger: f64
}

impl<'a> Listener<'a> {
    pub fn new(pinch_in_scale_trigger: f64, pinch_out_scale_trigger: f64, gesture_action: &dyn Fn(GestureType)) -> Listener {
        Listener {
            swipe: SwipeBuilder::empty(),
            pinch: PinchBuilder::empty(),
            pinch_in_scale_trigger,
            pinch_out_scale_trigger,
            gesture_action,
        }
    }

    pub fn event(&mut self, event: input::Event) {
        match event {
            Gesture(Swipe(Begin(event))) => self.swipe.new(event.finger_count()),
            Gesture(Swipe(Update(event))) => {
                self.swipe.update(event);
            }
            Gesture(Swipe(End(event))) => {
                match self.swipe.build(event) {
                    Ok(g) => match g.gesture_type() {
                        Some(t) => (self.gesture_action)(t),
                        None => warn!("cancelled or unrecognized gesture {:?}", g),
                    },
                    Err(s) => error!("no Gesture {:?}", s),
                }
            }

            Gesture(Pinch(GesturePinchEvent::Begin(event))) =>
                self.pinch.new(event.scale(), self.pinch_in_scale_trigger, self.pinch_out_scale_trigger),
            Gesture(Pinch(GesturePinchEvent::Update(event))) => {
                match self.pinch.update(&event) {
                    Some(p) => {
                        match p.gesture_type() {
                            Some(t) => (self.gesture_action)(t),
                            None => warn!("cancelled or unrecognized gesture {:?}", p)
                        };
                        self.pinch.new(event.scale(), self.pinch_in_scale_trigger, self.pinch_out_scale_trigger)
                    },
                    None => ()
                }
            }

            Gesture(Pinch(GesturePinchEvent::End(event))) => {
                let gesture = self.pinch.build(event);
                match gesture {
                    Ok(p) => match p.gesture_type() {
                        Some(t) => (self.gesture_action)(t),
                        None => warn!("cancelled or unrecognized gesture {:?}", p),
                    },
                    Err(s) => error!("no Gesture {:?}", s),
                }
            }

            _ => (),
        }
    }
}
