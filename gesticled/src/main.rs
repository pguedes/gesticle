extern crate clap;
extern crate libxdo;
extern crate libxdo_sys;
#[macro_use]
extern crate log;
extern crate gesticle;

use std::fs;
use std::os::raw::c_ulong;
use std::path::Path;
use std::ptr::null;
use std::sync::{Arc, Mutex};

use clap::{App, Arg};

use libxdo::XDo;
use libxdo_sys::xdo_free;
use libxdo_sys::xdo_get_active_window;
use libxdo_sys::xdo_get_pid_window;
use libxdo_sys::xdo_new;

use gesticle::gestures::{GestureType, gesture_channel};
use gesticle::configuration::{GestureActions, init_logging};
use gesticle::dbus;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct GestureHandler {
    actions: Arc<Mutex<GestureActions>>,
    xdo: XDo
}

impl GestureHandler {

    fn new(actions: Arc<Mutex<GestureActions>>) -> GestureHandler {
        let xdo = XDo::new(None).expect("failed to create xdo ctx");
        GestureHandler { xdo, actions }
    }

    fn handle(&self, t: GestureType) {

        let setting = self.context_sensitive_config(t.to_config().as_str()).
            or_else(|| self.get_setting(t.to_config().as_str()));

        match setting {
            Some(v) => {

                if v.is_empty() {
                    info!("skipping gesture due to no action: {:?}", t);
                } else {
                    self.xdo.send_keysequence(&v, 0).unwrap();
                }
            },
            None => warn!("gesture not configured: {:?}", t),
        }
    }

    fn current_window(&self) -> Result<String, String> {

        unsafe {
            let xdo = xdo_new(null());

            if xdo.is_null() {
                return Err("Failed to init libxdo.".to_owned());
            }

            let mut window: c_ulong = 0;

            if xdo_get_active_window(xdo, &mut window) != 0 {
                return Err("Failed to get window id".to_owned());
            }

            let pid = xdo_get_pid_window(xdo, window);

            xdo_free(xdo);

            let file = format!("/proc/{}/comm", pid);

            match fs::read_to_string(file) {
                Ok(name) => Ok(name.trim_end().to_owned()),
                Err(e) => Err(format!("failed to read process name: {:?}", e))
            }
        }
    }

    fn context_sensitive_config(&self, base: &str) -> Option<String> {

        match self.current_window() {
            Ok(window) => {
                let cfg = format!("{}.{}", window, base);
                self.get_setting(cfg.as_str())
            },
            Err(e) => {
                error!("could not detect current window: {:?}", e);
                None
            }
        }
    }

    fn get_setting(&self, setting: &str) -> Option<String> {
        let result = self.actions.lock().unwrap().get(setting);
        debug!("getting setting: {:?} = {:?}", setting, result);
        result
    }

}

fn main() {

    let args = App::new("gesticle").
        version(VERSION).
        author("pedro@guedes.pt").
        about("Configurable libinput gesture handling").
        arg(
            Arg::with_name("debug").long("debug").short("d").
                help("print debug information")
        ).
        arg(
            Arg::with_name("config").long("config").short("c").
                value_name("FILE.toml").
                validator(|f| {
                    if Path::new(&f).exists() {
                        Ok(())
                    } else {
                        Err("Config file not found".to_owned())
                    }
                }).
                help("use specific configuration file")
        ).
        get_matches();

    init_logging(args.is_present("debug"), None);

    let actions = GestureActions::new(args.value_of("config"));
    let pinch_in_scale_trigger = actions.get_float("gesture.trigger.pinch.in.scale").unwrap_or( 0.0);
    let pinch_out_scale_trigger = actions.get_float("gesture.trigger.pinch.out.scale").unwrap_or( 0.0);

    let actions_arc = Arc::new(Mutex::new(actions));

    dbus::server(actions_arc.clone());

    let handler = GestureHandler::new(actions_arc);

    for gesture in gesture_channel(pinch_in_scale_trigger, pinch_out_scale_trigger) {
        debug!("triggered gesture: {:?}", gesture);
        handler.handle(gesture);
    }
}

