use glib::Object;
use gtk::gio::ListStore;
use gtk::glib;

use gesticle::configuration::GestureActions;
use gesticle::gestures::{GestureType, PinchDirection, RotationDirection, SwipeDirection};

mod imp {
    use gdk::glib::subclass::basic;
    use gtk::subclass::prelude::*;

    use std::cell::RefCell;

    use super::*;

    #[derive(Default)]
    pub struct GestureSettingPrivate {
        config: RefCell<String>,
        direction: RefCell<String>,
        category: RefCell<String>,
        app: RefCell<Option<String>>,
        action: RefCell<Option<String>>,
        inherited: RefCell<Option<String>>,
        enabled: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GestureSettingPrivate {
        const NAME: &'static str = "GestureSetting";
        type Type = GestureSetting;
        type ParentType = Object;
        type Interfaces = ();
        type Instance = basic::InstanceStruct<Self>;
        type Class = basic::ClassStruct<Self>;
    }

    // // GObject property definitions for our two values
    // static PROPERTIES: [Property; 7] = [
    //     subclass::Property("config", |name| {
    //         glib::ParamSpec::string(
    //             name,
    //             "Config",
    //             "Config",
    //             None, // Default value
    //             glib::ParamFlags::READWRITE,
    //         )
    //     }),
    //     subclass::Property("direction", |name| {
    //         glib::ParamSpec::string(
    //             name,
    //             "Direction",
    //             "Direction",
    //             None, // Default value
    //             glib::ParamFlags::READWRITE,
    //         )
    //     }),
    //     subclass::Property("category", |name| {
    //         glib::ParamSpec::string(
    //             name,
    //             "Category",
    //             "Category",
    //             None, // Default value
    //             glib::ParamFlags::READWRITE,
    //         )
    //     }),
    //     subclass::Property("action", |name| {
    //         glib::ParamSpec::string(
    //             name,
    //             "Action",
    //             "Action",
    //             None, // Default value
    //             glib::ParamFlags::READWRITE,
    //         )
    //     }),
    //     subclass::Property("app", |name| {
    //         glib::ParamSpec::string(
    //             name,
    //             "App",
    //             "App",
    //             None, // Default value
    //             glib::ParamFlags::READWRITE,
    //         )
    //     }),
    //     subclass::Property("inherited", |name| {
    //         glib::ParamSpec::string(
    //             name,
    //             "Inherited Action",
    //             "Inherited Action",
    //             None,
    //             glib::ParamFlags::READWRITE,
    //         )
    //     }),
    //     subclass::Property("enabled", |name| {
    //         glib::ParamSpec::boolean(
    //             name,
    //             "Enabled",
    //             "Enabled",
    //             false,
    //             glib::ParamFlags::READWRITE,
    //         )
    //     }),
    // ];


    impl ObjectImpl for GestureSettingPrivate {

    }
}

glib::wrapper! {
        pub struct GestureSetting(ObjectSubclass<imp::GestureSettingPrivate>);
    }

impl GestureSetting {
    pub fn new_cfg(gesture_type: &GestureType, app: Option<&str>, config: &GestureActions) -> GestureSetting {
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
    ) -> Self {
        Object::new(&[
            ("config", &config),
            ("direction", &direction),
            ("category", &category),
            ("app", &app),
            ("action", &action),
            ("inherited", &inherited),
            ("enabled", &enabled)
        ])
            .expect("Failed to create row data")
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

    pub fn create_app_data(store: &ListStore, app: Option<&str>, config: &GestureActions) {
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
}
