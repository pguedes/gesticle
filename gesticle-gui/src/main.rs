use log::{error, info};

use std::env::args;
use std::collections::HashMap;

use glib::{clone, glib_object_subclass, glib_object_impl, glib_wrapper, glib_object_wrapper};
use gio::prelude::*;
use gio::ListStore;
use gtk::{
    Align, ApplicationWindow, BoxBuilder, Builder, Entry, LabelBuilder, ListBox, ListBoxRow,
    ListBoxRowBuilder, Orientation, SearchEntry, SwitchBuilder, SelectionMode, SearchBar,
    Button, EntryBuilder, ResponseType, Dialog,
};
use gtk::prelude::*;
use gdk::ModifierType;

use gesticle::gestures::{GestureType, PinchDirection, RotationDirection, SwipeDirection};
use gesticle::configuration::{GestureActions, home_path, init_logging};
use row_data::RowData;


pub fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("../gesticle-settings.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("app_window")
        .expect("Couldn't get app window");

    let list: ListBox = builder.get_object("listbox").expect("no listbox");

    list.set_selection_mode(SelectionMode::None);

    let model = gio::ListStore::new(RowData::static_type());

    list.bind_model(Some(&model), move |item| {
        let item: &RowData = item.downcast_ref::<RowData>().expect("wrong item type");

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

        let entry = EntryBuilder::new().visible(true).build();
        item.bind_property("inherited", &entry, "placeholder_text")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        item.bind_property("action", &entry, "text")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();

        entry.connect_key_press_event(|field, e| {
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

            Inhibit(true)
        });

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

    let setting_pinch_in_trigger_scale = &RowData::new(
        "gesture.trigger.pinch.in.scale".to_owned(),
        "".to_owned(),
        "settings".to_owned(),
        None,
        v1,
        None,
        true,
    ).upcast::<glib::Object>();
    let setting_pinch_out_trigger_scale = &RowData::new(
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
                    .margin_top(15)
                    .margin_bottom(10)
                    .build()
            ));
        }
    }))));

    let entry: SearchEntry = builder.get_object("action_filter").expect("no action filter");

    entry.connect_changed(clone!(@strong list => move |_| {
        list.invalidate_filter();
    }));

    list.set_filter_func(Some(Box::new(clone!(@strong model as store, @strong entry as s => move |row| {
        category(row, &store).to_lowercase()
            .contains(s.get_text().unwrap().to_lowercase().as_str())
    }))));

    window.set_application(Some(application));
    window.connect_delete_event(|win, _| {
        win.destroy();
        Inhibit(false)
    });

    let search_bar: SearchBar = builder.get_object("search_bar").expect("no search bar");

    window.connect_key_press_event(move |w, e| {

        // allow entry fields to get their events when focussed
        if let Some(focussed) = w.get_focus() {
            if focussed.is::<Entry>() {
                return Inhibit(false);
            }
        }

        let default_modifiers = gtk::accelerator_get_default_mod_mask();

        // quit when ctrl+w or ctrl+q is pressed
        if (e.get_state() & default_modifiers) == ModifierType::CONTROL_MASK &&
            (e.get_keyval() == gdk::enums::key::w || e.get_keyval() == gdk::enums::key::q) {
            w.close();
        }

        Inhibit(search_bar.handle_event(e))
    });

    let save_button: Button = builder.get_object("save_button").expect("no save button");
    save_button.connect_clicked(clone!(@strong model, @strong setting_pinch_out_trigger_scale, @strong setting_pinch_in_trigger_scale => move |_| {

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
                    Ok(_) => info!("configuration updated"),
                    Err(e) => error!("failed to update configuration: {:?}", e)
                }
            }
        }

    }));

    let add_button = builder.get_object::<Button>("add_button").expect("no add_button found");

    let dialog = builder.get_object::<Dialog>("add_app_dialog").expect("no add_app_dialog found");
    let app_entry = builder.get_object::<Entry>("add_app_entry").expect("no add_app_dialog found");

    add_button.connect_clicked(move |_| {
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
                }
            }
        }

        dialog.hide();
    });

    window.show_all();
}

fn category(row: &ListBoxRow, store: &gio::ListStore) -> String {
    let item = store.get_object(row.get_index() as u32).expect("no item on existing row");
    let category: String = item.get_property("category").unwrap()
        .get().expect("category property").unwrap();
    category
}

fn main() {
    init_logging(true, Some(".gesticle/gesticle-gui.log"));

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
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Up, 3), app, config));
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Down, 3), app, config));
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Left, 3), app, config));
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Right, 3), app, config));
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Up, 4), app, config));
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Down, 4), app, config));
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Left, 4), app, config));
    store.append(&RowData::new_cfg(&GestureType::Swipe(SwipeDirection::Right, 4), app, config));

    store.append(&RowData::new_cfg(&GestureType::Pinch(PinchDirection::In, 0.0), app, config));
    store.append(&RowData::new_cfg(&GestureType::Pinch(PinchDirection::Out, 0.0), app, config));

    store.append(&RowData::new_cfg(&GestureType::Rotation(RotationDirection::Left, 0.0), app, config));
    store.append(&RowData::new_cfg(&GestureType::Rotation(RotationDirection::Right, 0.0), app, config));
}

// Our GObject subclass for carrying a name and count for the ListBox model
//
// Both name and count are stored in a RefCell to allow for interior mutability
// and are exposed via normal GObject properties. This allows us to use property
// bindings below to bind the values with what widgets display in the UI
mod row_data {
    use glib::subclass;
    use glib::subclass::prelude::*;
    use glib::translate::*;

    use super::*;

    // Implementation sub-module of the GObject
    mod imp {
        use std::cell::RefCell;

        use super::*;

        // The actual data structure that stores our values. This is not accessible
        // directly from the outside.
        pub struct RowDataPrivate {
            config: RefCell<String>,
            direction: RefCell<String>,
            category: RefCell<String>,
            app: RefCell<Option<String>>,
            action: RefCell<Option<String>>,
            inherited: RefCell<Option<String>>,
            enabled: RefCell<bool>,
        }

        // GObject property definitions for our two values
        static PROPERTIES: [subclass::Property; 7] = [
            subclass::Property("config", |name| {
                glib::ParamSpec::string(
                    name,
                    "Config",
                    "Config",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("direction", |name| {
                glib::ParamSpec::string(
                    name,
                    "Direction",
                    "Direction",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("category", |name| {
                glib::ParamSpec::string(
                    name,
                    "Category",
                    "Category",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("action", |name| {
                glib::ParamSpec::string(
                    name,
                    "Action",
                    "Action",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("app", |name| {
                glib::ParamSpec::string(
                    name,
                    "App",
                    "App",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("inherited", |name| {
                glib::ParamSpec::string(
                    name,
                    "Inherited Action",
                    "Inherited Action",
                    None,
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("enabled", |name| {
                glib::ParamSpec::boolean(
                    name,
                    "Enabled",
                    "Enabled",
                    false,
                    glib::ParamFlags::READWRITE,
                )
            }),
        ];

        // Basic declaration of our type for the GObject type system
        impl ObjectSubclass for RowDataPrivate {
            const NAME: &'static str = "RowData";
            type ParentType = glib::Object;
            type Instance = subclass::simple::InstanceStruct<Self>;
            type Class = subclass::simple::ClassStruct<Self>;

            glib_object_subclass!();

            // Called exactly once before the first instantiation of an instance. This
            // sets up any type-specific things, in this specific case it installs the
            // properties so that GObject knows about their existence and they can be
            // used on instances of our type
            fn class_init(klass: &mut Self::Class) {
                klass.install_properties(&PROPERTIES);
            }

            // Called once at the very beginning of instantiation of each instance and
            // creates the data structure that contains all our state
            fn new() -> Self {
                Self {
                    config: RefCell::new("".to_string()),
                    direction: RefCell::new("".to_string()),
                    category: RefCell::new("".to_string()),
                    app: RefCell::new(None),
                    action: RefCell::new(None),
                    inherited: RefCell::new(None),
                    enabled: RefCell::new(false),
                }
            }
        }

        // The ObjectImpl trait provides the setters/getters for GObject properties.
        // Here we need to provide the values that are internally stored back to the
        // caller, or store whatever new value the caller is providing.
        //
        // This maps between the GObject properties and our internal storage of the
        // corresponding values of the properties.
        impl ObjectImpl for RowDataPrivate {
            glib_object_impl!();

            fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("config", ..) => {
                        let config = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`")
                            .unwrap();
                        self.config.replace(config);
                    }
                    subclass::Property("direction", ..) => {
                        let direction = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`")
                            .unwrap();
                        self.direction.replace(direction);
                    }
                    subclass::Property("category", ..) => {
                        let category = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`")
                            .unwrap();
                        self.category.replace(category);
                    }
                    subclass::Property("app", ..) => {
                        let action = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.app.replace(action);
                    }
                    subclass::Property("action", ..) => {
                        let action = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.action.replace(action);
                    }
                    subclass::Property("enabled", ..) => {
                        let enabled = value
                            .get_some()
                            .expect("type conformity checked by `Object::set_property`");
                        self.enabled.replace(enabled);
                    }
                    subclass::Property("inherited", ..) => {
                        let inherited = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.inherited.replace(inherited);
                    }
                    _ => unimplemented!(),
                }
            }

            fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("config", ..) => Ok(self.config.borrow().to_value()),
                    subclass::Property("direction", ..) => Ok(self.direction.borrow().to_value()),
                    subclass::Property("category", ..) => Ok(self.category.borrow().to_value()),
                    subclass::Property("app", ..) => Ok(self.app.borrow().to_value()),
                    subclass::Property("action", ..) => Ok(self.action.borrow().as_ref().or(Some(&"".to_owned())).to_value()),
                    subclass::Property("inherited", ..) => Ok(self.inherited.borrow().to_value()),
                    subclass::Property("enabled", ..) => Ok(self.enabled.borrow().to_value()),
                    _ => unimplemented!(),
                }
            }
        }
    }

    // Public part of the RowData type. This behaves like a normal gtk-rs-style GObject
    // binding
    glib_wrapper! {
        pub struct RowData(Object<subclass::simple::InstanceStruct<imp::RowDataPrivate>, subclass::simple::ClassStruct<imp::RowDataPrivate>, RowDataClass>);

        match fn {
            get_type => || imp::RowDataPrivate::get_type().to_glib(),
        }
    }

    // Constructor for new instances. This simply calls glib::Object::new() with
    // initial values for our two properties and then returns the new instance
    impl RowData {
        pub fn new_cfg(gesture_type: &GestureType, app: Option<&str>, config: &GestureActions) -> RowData {
            let setting = &gesture_type.to_config();
            let inherited = config.get_for_app(setting, None);

            let action = if config.is_specified(setting, app) {
                config.get_for_app(setting, app)
            } else {
                None
            };

            Self::new(
                GestureActions::key_for_app(gesture_type.to_config(), app),
                Self::gesture_direction(gesture_type),
                Self::gesture_category(gesture_type, app),
                app,
                action,
                inherited,
                !config.is_disabled(setting, app),
            )
        }

        pub fn new(
            config: String,
            direction: String,
            category: String,
            app: Option<&str>,
            action: Option<String>,
            inherited: Option<String>,
            enabled: bool,
        ) -> RowData {
            glib::Object::new(Self::static_type(), &[
                ("config", &config),
                ("direction", &direction),
                ("category", &category),
                ("app", &app),
                ("action", &action),
                ("inherited", &inherited),
                ("enabled", &enabled)
            ])
                .expect("Failed to create row data")
                .downcast()
                .expect("Created row data is of wrong type")
        }

        fn gesture_direction(gesture_type: &GestureType) -> String {
            match gesture_type {
                GestureType::Swipe(direction, _) => direction.to_string(),
                GestureType::Rotation(direction, _) => direction.to_string(),
                GestureType::Pinch(direction, _) => direction.to_string()
            }
        }

        fn gesture_category(gesture_type: &GestureType, app: Option<&str>) -> String {
            let mut category = match gesture_type {
                GestureType::Swipe(_, fingers) => format!("{} fingers Swipes", fingers),
                GestureType::Rotation(_, _) => "Rotations".to_owned(),
                GestureType::Pinch(_, _) => "Pinches".to_owned()
            };

            if let Some(context) = app {
                category.push_str(format!(" in {}", context).as_str());
            }
            category
        }
    }
}