extern crate gio;
extern crate gtk;
extern crate udev;
extern crate input;
extern crate libc;
extern crate nix;
#[macro_use]
extern crate log;

mod events;
mod gestures;

use std::env::args;
use events::listen;
use gestures::GestureType;

use gio::prelude::*;
use gtk::{ApplicationWindow, Builder, Label};
use gtk::prelude::*;


pub fn build_ui(application: &gtk::Application) {

    let glade_src = include_str!("../gesticle-settings.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("app_window").
        expect("Couldn't get app window");

    let label: Label = builder.get_object("gesture-label").
        expect("cannot find label");

    window.set_application(application);
    window.connect_delete_event(|win, _| {
        win.destroy();
        Inhibit(false)
    });

    window.show_all();

//    listen(|e| label.set_markup("<small>Small text</small>"));
}

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