extern crate config;
extern crate udev;
extern crate input;
extern crate libc;
extern crate libxdo;
extern crate libxdo_sys;
#[macro_use]
extern crate log;
extern crate nix;
extern crate simplelog;
extern crate clap;
extern crate dirs;

use clap::{Arg, App};

use std::fs;
use std::fs::File;
use std::os::raw::c_ulong;
use std::ptr::null;
use std::path::Path;

use libxdo::XDo;
use libxdo_sys::xdo_free;
use libxdo_sys::xdo_get_active_window;
use libxdo_sys::xdo_get_pid_window;
use libxdo_sys::xdo_new;

use simplelog::*;
use events::listen;
use std::path::PathBuf;
use gestures::GestureType;
use std::fs::create_dir;

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

    fn new(path: PathBuf) -> GestureHandler {
        let xdo = XDo::new(None).expect("failed to create xdo ctx");
        let mut settings = config::Config::new();
        settings.merge(config::File::from(path)).unwrap();
        GestureHandler { xdo, settings }
    }

    fn handle(&self, t: GestureType) {

        let setting = self.context_sensitive_config(t.to_config().as_str()).
            or_else(|| self.get_setting(t.to_config().as_str()));

        match setting {
            Some(v) => self.xdo.send_keysequence(&v, 0).unwrap(),
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

    let handler = GestureHandler::new(config_file_path);

    listen(|t| handler.handle(t));
}

