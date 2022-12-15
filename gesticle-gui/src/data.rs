use glib::Object;
use gtk::gio::ListStore;
use gtk::glib;

use gesticle::configuration::GestureActions;
use gesticle::gestures::{GestureType, PinchDirection, RotationDirection, SwipeDirection};

mod imp {
    use std::cell::RefCell;

    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecString, ParamSpecBoolean, ToValue, Value};
    use gtk::glib::once_cell::sync::Lazy;
    use gtk::subclass::prelude::*;

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
    }

    impl ObjectImpl for GestureSettingPrivate {

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("config", "Config", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("direction", "Direction", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("category", "Category", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("action", "Action", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("app", "App", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("inherited", "Inherited Action", "", None, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("enabled", "Enabled", "", true, ParamFlags::READWRITE)
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "config" => {
                    let config = value.get().unwrap();
                    self.config.replace(config);
                }
                "direction" => {
                    let direction = value.get().unwrap();
                    self.direction.replace(direction);
                }
                "category" => {
                    let category = value.get().unwrap();
                    self.category.replace(category);
                }
                "action" => {
                    let action = value.get().unwrap();
                    self.action.replace(action);
                }
                "app" => {
                    let app = value.get().unwrap();
                    self.app.replace(app);
                }
                "inherited" => {
                    let inherited = value.get().unwrap();
                    self.inherited.replace(inherited);
                }
                "enabled" => {
                    let enabled = value.get().unwrap();
                    self.enabled.replace(enabled);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "config" => self.config.borrow().to_value(),
                "direction" => self.direction.borrow().to_value(),
                "category" => self.category.borrow().to_value(),
                "action" => self.action.borrow().to_value(),
                "app" => self.app.borrow().to_value(),
                "inherited" => self.inherited.borrow().to_value(),
                "enabled" => self.enabled.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
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
