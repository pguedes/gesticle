mod data;

use log::{error, info};

use std::env::args;
use std::collections::HashMap;
use std::time::Duration;

use glib::clone;
use gio::prelude::*;
use gio::ListStore;
use gtk::{
    Align, ApplicationWindow, BoxBuilder, Builder, Entry, LabelBuilder, ListBox, ListBoxRow,
    ListBoxRowBuilder, Orientation, SearchEntry, SwitchBuilder, SelectionMode, SearchBar, ToggleButton,
    Button, EntryBuilder, ResponseType, Dialog, MessageDialog, DialogFlags, MessageType, ButtonsType
};
use gtk::prelude::*;
use gdk::ModifierType;

use gesticle::gestures::{GestureType, PinchDirection, RotationDirection, SwipeDirection};
use gesticle::configuration::{GestureActions, home_path, init_logging};

use data::GestureSetting;

use dbus::blocking::Connection;

pub fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("../gesticle-settings.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("app_window")
        .expect("Couldn't get app window");

    let list: ListBox = builder.get_object("listbox").expect("no listbox");

    list.set_selection_mode(SelectionMode::None);

    let model = gio::ListStore::new(GestureSetting::static_type());

    let manual_input_button : ToggleButton = builder.get_object("manual_input")
        .expect("no manual input toggle");

    list.bind_model(Some(&model), move |item| {
        let item: &GestureSetting = item.downcast_ref::<GestureSetting>().expect("wrong item type");

        let row = ListBoxRowBuilder::new()
            .selectable(false)
            .activatable(false)
            .visible(true)
            .margin_bottom(10)
            .build();

        let row_item_box = BoxBuilder::new()
            .orientation(Orientation::Horizontal)
            .halign(Align::Center)
            .homogeneous(true)
            .visible(true)
            .spacing(20)
            .build();

        let direction = LabelBuilder::new().visible(true).halign(Align::End).build();
        item.bind_property("direction", &direction, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        let entry = EntryBuilder::new()
            .secondary_icon_name("edit-clear-symbolic")
            .visible(true)
            .build();

        entry.connect_icon_press(|e,_,_| e.set_text(""));

        item.bind_property("inherited", &entry, "placeholder_text")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        item.bind_property("action", &entry, "text")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        entry.connect_key_press_event(clone!(@strong manual_input_button => move |field, e| {

            let manual_input = manual_input_button.get_active();

            if !manual_input {
                let mut name = "".to_owned();
                if e.get_state().contains(ModifierType::CONTROL_MASK) {
                    name.push_str("ctrl+")
                }
                if e.get_state().contains(ModifierType::MOD1_MASK) {
                    name.push_str("alt+")
                }
                if e.get_state().contains(ModifierType::SHIFT_MASK) {
                    name.push_str("shift+")
                }
                if e.get_state().contains(ModifierType::SUPER_MASK) {
                    name.push_str("super+")
                }
                name.push_str(gdk::keyval_name(e.get_keyval()).as_deref().expect("no name?"));

                field.set_text(name.as_str());
            }
            Inhibit(!manual_input)
        }));

        let switch = SwitchBuilder::new().visible(true).halign(Align::Start).valign(Align::Center).build();
        item.bind_property("enabled", &switch, "active")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        row_item_box.add(&direction);
        row_item_box.add(&entry);
        row_item_box.add(&switch);

        row.add(&row_item_box);

        row.upcast::<gtk::Widget>()
    });

    let actions = GestureActions::new(None); // TODO we should allow selecting which file to edit!
    create_app_data(&model, None, &actions);
    for app in actions.apps().unwrap() {
        create_app_data(&model, Some(app.as_str()), &actions);
    }

    let v1: Option<String> = match actions.get_float("gesture.trigger.pinch.in.scale") {
        Some(t) => Some(t.to_string()),
        None => None
    };
    let v2: Option<String> = match actions.get_float("gesture.trigger.pinch.out.scale") {
        Some(t) => Some(t.to_string()),
        None => None
    };

    let setting_pinch_in_trigger_scale = &GestureSetting::new(
        "gesture.trigger.pinch.in.scale".to_owned(),
        "".to_owned(),
        "settings".to_owned(),
        None,
        v1,
        None,
        true,
    ).upcast::<glib::Object>();
    let setting_pinch_out_trigger_scale = &GestureSetting::new(
        "gesture.trigger.pinch.out.scale".to_owned(),
        "".to_owned(),
        "settings".to_owned(),
        None,
        v2,
        None,
        true,
    ).upcast::<glib::Object>();

    let pinch_out_trigger_entry: Entry = builder.get_object("pinch_out_trigger").unwrap();
    let pinch_in_trigger_entry: Entry = builder.get_object("pinch_in_trigger").unwrap();

    setting_pinch_out_trigger_scale.bind_property("action", &pinch_out_trigger_entry, "text")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
        .build();
    setting_pinch_in_trigger_scale.bind_property("action", &pinch_in_trigger_entry, "text")
        .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
        .build();

    list.set_header_func(Some(Box::new(clone!(@weak model as store => move |row, prev| {

        let item_category = category(row, &store);

        let show_header = prev.map(|r| category(r, &store)).or(Some("".to_owned()))
            .filter(|c| !item_category.eq(c))
            .is_some();

        if show_header {
            row.set_header(Some(
                &LabelBuilder::new()
                    .label(format!("<span size=\"larger\" weight=\"bold\">{}</span>", item_category).as_str())
                    .use_markup(true)
                    .visible(true)
                    .margin_top(15)
                    .margin_bottom(10)
                    .build()
            ));
        }
    }))));

    let filter_entry: SearchEntry = builder.get_object("action_filter").expect("no action filter");

    filter_entry.connect_changed(clone!(@strong list => move |_| {
        list.invalidate_filter();
    }));

    list.set_filter_func(Some(Box::new(clone!(@strong model as store, @strong filter_entry as s => move |row| {
        category(row, &store).to_lowercase()
            .contains(s.get_text().unwrap().to_lowercase().as_str())
    }))));

    window.set_application(Some(application));
    window.connect_delete_event(|win, _| {
        win.destroy();
        Inhibit(false)
    });

    let save_button: Button = builder.get_object("save_button").expect("no save button");
    save_button.connect_clicked(clone!(@strong window, @strong model, @strong setting_pinch_out_trigger_scale, @strong setting_pinch_in_trigger_scale => move |_| {
        save(&model, &setting_pinch_in_trigger_scale, &setting_pinch_out_trigger_scale, &window);
    }));

    let add_button = builder.get_object::<Button>("add_button").expect("no add_button found");

    let dialog = builder.get_object::<Dialog>("add_app_dialog").expect("no add_app_dialog found");
    let app_entry = builder.get_object::<Entry>("add_app_entry").expect("no add_app_dialog found");

    let search_bar: SearchBar = builder.get_object("search_bar").expect("no search bar");

    add_button.connect_clicked(clone!(@strong filter_entry, @strong search_bar => move |_| {
        add(&app_entry, &dialog, &model, &actions, &filter_entry, &search_bar);
    }));

    window.connect_key_press_event(clone!(@strong search_bar => move |w, e| {

        // allow entry fields to get their events when focussed
        if let Some(focussed) = w.get_focus() {
            if focussed.is::<Entry>() {
                return Inhibit(false);
            }
        }

        let default_modifiers = gtk::accelerator_get_default_mod_mask();
        let control_pressed = (e.get_state() & default_modifiers) == ModifierType::CONTROL_MASK;
        // quit when ctrl+w or ctrl+q is pressed
        if control_pressed && (e.get_keyval() == gdk::enums::key::w || e.get_keyval() == gdk::enums::key::q) {
            w.close();
        } else if control_pressed && e.get_keyval() == gdk::enums::key::s {
            save_button.clicked();
        } else if control_pressed && e.get_keyval() == gdk::enums::key::a {
            add_button.clicked();
        }
        Inhibit(search_bar.handle_event(e))
    }));

    window.show_all();
}

fn category(row: &ListBoxRow, store: &gio::ListStore) -> String {
    let item = store.get_object(row.get_index() as u32).expect("no item on existing row");
    let category: String = item.get_property("category").unwrap()
        .get().expect("category property").unwrap();
    category
}

fn add(
    app_entry: &Entry,
    dialog: &Dialog,
    model: &gio::ListStore,
    actions: &GestureActions,
    filter_entry: &SearchEntry,
    search_bar: &SearchBar
) {
    app_entry.set_text("");
    app_entry.grab_focus();

    if ResponseType::Apply == dialog.run() {
        let app = app_entry.get_text();
        let mut exists = false;
        if let Some(new_app) = app {
            for index in 0..model.get_n_items() {
                let item = model.get_object(index as u32).expect("no item on existing row");
                let model_app = item.get_property("app").unwrap()
                    .get::<String>().expect("app property");

                if let Some(existent) = model_app {
                    if new_app == existent {
                        exists = true;
                    }
                }
            }
            if !exists {
                create_app_data(&model, Some(new_app.as_str()), &actions);
                filter_entry.set_text(new_app.as_str());
                search_bar.set_search_mode(true);
            }
        }
    }
    dialog.hide();
}

fn save(
    model: &gio::ListStore,
    setting_pinch_in_trigger_scale: &glib::Object,
    setting_pinch_out_trigger_scale: &glib::Object,
    window: &gtk::ApplicationWindow
) {

    let mut actions = HashMap::new();

    let append_item = |actions: &mut HashMap<String, HashMap<String, String>>, item: &glib::Object| {
        let config = item.get_property("config").unwrap()
            .get::<String>().expect("config property").unwrap();

        let enabled = item.get_property("enabled").unwrap()
            .get::<bool>().expect("enabled property").unwrap();
        let action = if enabled {
            item.get_property("action").unwrap().get::<String>().expect("action property")
        } else {
            Some("".to_owned())
        };

        if let Some(value) = action.filter(|s| !s.is_empty()) {

            let mut parts = config.split('.');
            let key = parts.next_back().unwrap().to_owned();
            let table = parts.collect::<Vec<&str>>().join(".");

            let table_actions = actions.entry(table).or_insert(HashMap::new());
            (*table_actions).insert(key, value);
        }
    };

    for index in 0..model.get_n_items() {
        let item = model.get_object(index as u32).expect("no item on existing row");
        append_item(&mut actions, &item);
    }

    append_item(&mut actions, &setting_pinch_out_trigger_scale);
    append_item(&mut actions, &setting_pinch_in_trigger_scale);

    let s = toml::to_string_pretty(&actions).unwrap();

    if let Ok(_) = std::fs::write("/tmp/crap.toml", &s) {
        if let Some(home_config_file) = home_path(".gesticle/config.toml") {
            match std::fs::rename("/tmp/crap.toml", home_config_file) {
                Ok(_) => {
                    let dbus = Connection::new_session().unwrap();
                    let proxy = dbus.with_proxy("io.github.pguedes.gesticle", "/actions/reload", Duration::from_millis(5000));
                    match proxy.method_call("io.github.pguedes.gesticle", "reload", ()) {
                        Ok(()) => {
                            let msg = MessageDialog::new(Some(window), DialogFlags::MODAL, MessageType::Info,
                                                         ButtonsType::Ok, "Configuration updated");
                            msg.run();
                            msg.hide();
                            info!("configuration updated");
                        },
                        Err(e) => {
                            let msg = MessageDialog::new(Some(window), DialogFlags::MODAL, MessageType::Error,
                                                         ButtonsType::Ok, "Configuration file updated but could not call daemon to update runtime configuration... is it running?");
                            msg.run();
                            msg.hide();
                            error!("failed to update runtime configuration: {:?}", e)
                        }
                    }
                },
                Err(e) => error!("failed to update configuration: {:?}", e)
            }
        }
    }
}

fn main() {
    init_logging(false, Some(".gesticle/gesticle-gui.log"));

    let application =
        gtk::Application::new(Some("pt.guedes.gesticle-settings-gui"), gio::ApplicationFlags::empty())
            .expect("Initialization failed...");

    application.connect_startup(|app| {
        build_ui(app);
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}

fn create_app_data(store: &ListStore, app: Option<&str>, config: &GestureActions) {
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Up, 3), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Down, 3), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Left, 3), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Right, 3), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Up, 4), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Down, 4), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Left, 4), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Right, 4), app, config));

    store.append(&GestureSetting::new_cfg(&GestureType::Pinch(PinchDirection::In, 0.0), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Pinch(PinchDirection::Out, 0.0), app, config));

    store.append(&GestureSetting::new_cfg(&GestureType::Rotation(RotationDirection::Left, 0.0), app, config));
    store.append(&GestureSetting::new_cfg(&GestureType::Rotation(RotationDirection::Right, 0.0), app, config));
}