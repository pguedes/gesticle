mod data;

use log::{error, info};

use std::env::args;
use std::collections::HashMap;

use glib::clone;
use gio::prelude::*;
use gio::ListStore;
use gtk::{
    Align, ApplicationWindow, BoxBuilder, Builder, Entry, LabelBuilder, ListBox, Button,
    ListBoxRowBuilder, Orientation, SearchEntry, SwitchBuilder, SearchBar, ToggleButton,
    EntryBuilder, ResponseType, Dialog, MessageDialog, DialogFlags, MessageType, ButtonsType,
};
use gtk::prelude::*;
use gdk::ModifierType;

use gesticle::configuration::{GestureActions, home_path, init_logging};
use gesticle::dbus;

use data::GestureSetting;
use std::rc::Rc;

struct GesticleGui {
    actions: Rc<GestureActions>,
    data_store: ListStore,
    window: ApplicationWindow,
    save_button: Button,
    add_button: Button,
    filter_entry: SearchEntry,
    dialog: Dialog,
    app_entry: Entry,
    search_bar: SearchBar,
    list: ListBox,
    manual_input_button: ToggleButton,
    pinch_out_trigger_entry: Entry,
    pinch_in_trigger_entry: Entry,
    setting_pinch_out_trigger_scale: glib::Object,
    setting_pinch_in_trigger_scale: glib::Object,
}

impl GesticleGui {
    fn from_builder(builder: &Builder, actions: GestureActions, application: &gtk::Application) -> GesticleGui {

        let pinch_in_value = match actions.get_float("gesture.trigger.pinch.in.scale") {
            Some(t) => Some(t.to_string()),
            None => None
        };
        let pinch_out_value = match actions.get_float("gesture.trigger.pinch.out.scale") {
            Some(t) => Some(t.to_string()),
            None => None
        };

        let gui = GesticleGui {
            actions: Rc::new(actions),
            data_store: gio::ListStore::new(GestureSetting::static_type()),
            window: builder.get_object("app_window").expect("Couldn't get app window"),
            save_button: builder.get_object("save_button").expect("no save button"),
            add_button: builder.get_object("add_button").expect("no add_button found"),
            filter_entry: builder.get_object("action_filter").expect("no action filter"),
            dialog: builder.get_object::<Dialog>("add_app_dialog").expect("no add_app_dialog found"),
            app_entry: builder.get_object::<Entry>("add_app_entry").expect("no add_app_dialog found"),
            search_bar: builder.get_object("search_bar").expect("no search bar"),
            list: builder.get_object("listbox").expect("no listbox"),
            manual_input_button: builder.get_object("manual_input").expect("no manual input toggle"),
            pinch_out_trigger_entry: builder.get_object("pinch_out_trigger").unwrap(),
            pinch_in_trigger_entry: builder.get_object("pinch_in_trigger").unwrap(),
            setting_pinch_in_trigger_scale: GestureSetting::new(
                "gesture.trigger.pinch.in.scale".to_owned(),
                "".to_owned(),
                "settings".to_owned(),
                None,
                pinch_in_value,
                None,
                true,
            ).upcast::<glib::Object>(),
            setting_pinch_out_trigger_scale: GestureSetting::new(
                "gesture.trigger.pinch.out.scale".to_owned(),
                "".to_owned(),
                "settings".to_owned(),
                None,
                pinch_out_value,
                None,
                true,
            ).upcast::<glib::Object>(),
        };

        gui.window.set_application(Some(application));

        gui.connect_gui_events();
        gui.bind_data();

        gui
    }

    fn show_all(&self) {
        self.window.show_all();
    }

    fn category(index: u32, store: &gio::ListStore) -> String {
        let item = store.get_object(index).expect("no item on existing row");
        let category: String = item.get_property("category").unwrap()
            .get().expect("category property").unwrap();
        category
    }

    /// Add actions for a specific application
    fn add(
        app_entry: &Entry,
        dialog: &Dialog,
        model: &gio::ListStore,
        actions: &GestureActions,
        filter_entry: &SearchEntry,
        search_bar: &SearchBar,
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
                    GestureSetting::create_app_data(&model, Some(new_app.as_str()), &actions);
                    filter_entry.set_text(new_app.as_str());
                    search_bar.set_search_mode(true);
                }
            }
        }
        dialog.hide();
    }

    /// Save current actions to filesystem and let gesticled (daemon) know to re-load configuration
    fn save(
        model: &gio::ListStore,
        setting_pinch_in_trigger_scale: &glib::Object,
        setting_pinch_out_trigger_scale: &glib::Object,
        window: &gtk::ApplicationWindow,
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
                        match dbus::config_update() {
                            Ok(()) => {
                                let msg = MessageDialog::new(Some(window), DialogFlags::MODAL, MessageType::Info,
                                                             ButtonsType::Ok, "Configuration updated");
                                msg.run();
                                msg.hide();
                                info!("configuration updated");
                            }
                            Err(e) => {
                                let msg = MessageDialog::new(Some(window), DialogFlags::MODAL, MessageType::Error,
                                                             ButtonsType::Ok, "Configuration file updated but could not call daemon to update runtime configuration... is it running?");
                                msg.run();
                                msg.hide();
                                error!("failed to update runtime configuration: {:?}", e)
                            }
                        }
                    }
                    Err(e) => error!("failed to update configuration: {:?}", e)
                }
            }
        }
    }

    fn bind_data(&self) {
        self.list.bind_model(Some(&self.data_store), clone!(@strong self.manual_input_button as manual_input_button => move |item| {
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

            entry.connect_icon_press(|e, _, _| e.set_text(""));

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
        }));

        GestureSetting::create_app_data(&self.data_store, None, &self.actions);
        for app in self.actions.apps().unwrap() {
            GestureSetting::create_app_data(&self.data_store, Some(app.as_str()), &self.actions);
        }

        self.setting_pinch_out_trigger_scale.bind_property("action", &self.pinch_out_trigger_entry, "text")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
        self.setting_pinch_in_trigger_scale.bind_property("action", &self.pinch_in_trigger_entry, "text")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
    }

    fn connect_gui_events(&self) {
        self.window.connect_delete_event(|win, _| {
            win.destroy();
            Inhibit(false)
        });

        self.add_button.connect_clicked(
            clone!(@strong self.filter_entry as filter_entry, @strong self.search_bar as search_bar,
                        @strong self.app_entry as app_entry, @strong self.dialog as dialog,
                        @strong self.data_store as data_store, @strong self.actions as actions => move |_| {
                Self::add(&app_entry, &dialog, &data_store, &actions, &filter_entry, &search_bar);
            }
        ));

        self.window.connect_key_press_event(clone!(@strong self.search_bar as search_bar, @strong self.save_button as save_button, @strong self.add_button as add_button => move |w, e| {

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

        self.list.set_header_func(Some(Box::new(clone!(@weak self.data_store as store => move |row, prev| {

            let item_category = Self::category(row.get_index() as u32, &store);

            let show_header = prev.map(|r| Self::category(r.get_index() as u32, &store)).or(Some("".to_owned()))
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

        self.filter_entry.connect_changed(clone!(@strong self.list as list => move |_| {
            list.invalidate_filter();
        }));

        self.list.set_filter_func(Some(Box::new(clone!(@strong self.data_store as store, @strong self.filter_entry as s => move |row| {
            Self::category(row.get_index() as u32, &store).to_lowercase()
                .contains(s.get_text().unwrap().to_lowercase().as_str())
        }))));

        self.save_button.connect_clicked(
            clone!(@strong self.window as window, @strong self.data_store as model,
                        @strong self.setting_pinch_out_trigger_scale as setting_pinch_out_trigger_scale,
                        @strong self.setting_pinch_in_trigger_scale as setting_pinch_in_trigger_scale => move |_| {
                Self::save(&model, &setting_pinch_in_trigger_scale, &setting_pinch_out_trigger_scale, &window);
            })
        );
    }
}

fn main() {
    init_logging(false, Some(".gesticle/gesticle-gui.log"));

    let application =
        gtk::Application::new(Some("pt.guedes.gesticle-settings-gui"), gio::ApplicationFlags::empty())
            .expect("Initialization failed...");

    application.connect_startup(|app| {
        let glade_src = include_str!("../gesticle-settings.glade");
        let builder = Builder::new_from_string(glade_src);
        let actions = GestureActions::new(None); // TODO we should allow selecting which file to edit!

        let gui = GesticleGui::from_builder(&builder, actions, app);

        gui.show_all();
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
