extern crate gio;
extern crate gtk;
extern crate glib;
extern crate udev;
extern crate input;
extern crate libc;
extern crate nix;
#[macro_use]
extern crate log;
extern crate dirs;
extern crate config;

mod events;
mod gestures;
mod paths;

mod gui { pub mod settings; }

use std::env::args;
use events::listen;
use gestures::GestureType;

use gio::prelude::*;
use gtk::{ApplicationWindow, Builder, Label, Grid};
use gtk::prelude::*;
use std::thread;
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver};


pub fn build_ui(application: &gtk::Application) {

    let glade_src = include_str!("../gesticle-settings.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("app_window").
        expect("Couldn't get app window");

    let settings_grid : Grid = builder.get_object("settings-grid").
        expect("Could not get settings grid");

    let settings = gui::settings::Settings::new(settings_grid).unwrap();
    settings.build_gui();

    let label: Label = builder.get_object("gesture-label").
        expect("cannot find label");

    window.set_application(application);
    window.connect_delete_event(|win, _| {
        win.destroy();
        Inhibit(false)
    });

    window.show_all();

    let (tx, rx) = channel();

    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((label, rx));
    });

    thread::spawn(move || {
        listen(|e| {
            println!("event got {:?}", e);
            tx.send(e).unwrap();
            glib::idle_add(event);
        });
    });
}

fn event() -> glib::Continue {
    GLOBAL.with(|global| {
        if let Some((ref label, ref rx)) = *global.borrow() {
            if let Ok(event) = rx.try_recv() {
                label.set_text(&format!("event {:?}", event));
            }
        }
    });

    glib::Continue(false)
}

// declare a new thread local storage key
thread_local!(
    static GLOBAL: RefCell<Option<(gtk::Label, Receiver<GestureType>)>> = RefCell::new(None)
);

fn main() {

    let application = gtk::Application::new("pt.guedes.gesticle-settings-gui",
                                            gio::ApplicationFlags::empty()).
        expect("Initialization failed...");

    application.connect_startup(|app| {
        build_ui(app);
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}