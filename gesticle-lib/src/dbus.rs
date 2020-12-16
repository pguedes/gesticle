use std::thread;
use std::sync::{Mutex, Arc};
use std::time::Duration;

use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;

use crate::configuration::GestureActions;
use dbus::Error;

/// Creates a dbus-server in a new thread to allow reloading configuration when called
pub fn server(actions_ref: Arc<Mutex<GestureActions>>) {
    thread::spawn(move || {
        let c = Connection::new_session().expect("d-bus session");
        c.request_name("io.github.pguedes.gesticle", false, true, false).expect("d-bus name");
        let mut cr = Crossroads::new();
        let token = cr.register("io.github.pguedes.gesticle", move |b| {
            b.method("reload", (), (), move |_, _, _: ()| {
                let mut actions = actions_ref.lock().unwrap();
                actions.reload();
                debug!("actions reloaded: {:?}", actions);
                Ok(())
            });
        });
        cr.insert("/actions/reload", &[token], ());
        cr.serve(&c).expect("d-bus serve");
    });
}

/// Call d-bus endpoint to request a configuration update
pub fn config_update() -> Result<(), Error> {
    let dbus = Connection::new_session().unwrap();
    let proxy = dbus.with_proxy("io.github.pguedes.gesticle", "/actions/reload", Duration::from_millis(5000));
    proxy.method_call("io.github.pguedes.gesticle", "reload", ())
}