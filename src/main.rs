extern crate clap;
extern crate config;
extern crate dirs;
extern crate input;
extern crate libc;
extern crate libxdo;
extern crate libxdo_sys;
#[macro_use]
extern crate log;
extern crate nix;
extern crate simplelog;
extern crate udev;

use std::fs;
use std::fs::create_dir;
use std::fs::File;
use std::os::raw::c_ulong;
use std::path::Path;
use std::path::PathBuf;
use std::ptr::null;

use clap::{App, Arg};
use libxdo::XDo;
use libxdo_sys::xdo_free;
use libxdo_sys::xdo_get_active_window;
use libxdo_sys::xdo_get_pid_window;
use libxdo_sys::xdo_new;
use simplelog::*;

use events::GestureSource;
use gestures::GestureType;

mod gestures;
mod events;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn init_logging(debug: bool) {

    let log_path = home_path(".gesticle/gesticle.log").
        expect("cannot create log file path");

    if debug {
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Debug, Config::default()).unwrap(),
                WriteLogger::new(LevelFilter::Debug, Config::default(),
                                 File::create(log_path).unwrap()),
            ]
        ).unwrap();
    } else {
        WriteLogger::init(LevelFilter::Info, Config::default(),
                         File::create(log_path).unwrap()).unwrap();
    }
}

fn home_path(relative_path: &str) -> Option<PathBuf> {
    match dirs::home_dir() {
        Some(dir) => Some(dir.join(Path::new(relative_path))),
        None => None
    }
}

fn config_file_path(config_path_override: Option<&str>) -> Result<PathBuf, &str> {

    let path_exists = |p: &PathBuf| p.exists();

    match config_path_override {
        Some(o) => Ok(Path::new(o).to_owned()),
        None => {
            home_path(".gesticle/config.toml").
                filter(path_exists).
                or(Some(Path::new("/etc/gesticle/config.toml").to_owned())).
                filter(path_exists).
                ok_or("nothing in ~/.gesticle/config.toml or /etc/gesticle/config.toml")
        }
    }
}

struct GestureHandler {
    settings: config::Config,
    xdo: XDo
}

impl GestureHandler {

    fn new(settings: config::Config) -> GestureHandler {
        let xdo = XDo::new(None).expect("failed to create xdo ctx");
        GestureHandler { xdo, settings }
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

        let result = self.settings.get_str(setting);
        debug!("getting setting: {:?} = {:?}", setting, result);

        match result {
            Ok(val) => Some(val),
            Err(_) => None
        }
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

    let user_app_home = home_path(".gesticle").expect("cannot find user home");
    if !user_app_home.exists() {
        create_dir(user_app_home).expect("cannot create app dir in user home");
    }

    init_logging(args.is_present("debug"));

    let config_file_path = config_file_path(args.value_of("config")).
            expect("config file not found");

    info!("creating handler from configuration: {:?}", config_file_path);

    let mut settings = config::Config::new();
    settings.merge(config::File::from(config_file_path)).unwrap();

    let pinch_in_scale_trigger = settings.get_float("gesture.trigger.pinch.in.scale")
        .unwrap_or( 0.0);
    let pinch_out_scale_trigger = settings.get_float("gesture.trigger.pinch.out.scale")
        .unwrap_or( 0.0);

    let gesture_source = GestureSource::new(pinch_in_scale_trigger, pinch_out_scale_trigger);

    let handler = GestureHandler::new(settings);

    for gesture in gesture_source.listen() {
        debug!("triggered gesture: {:?}", gesture);
        handler.handle(gesture);
    }
}

