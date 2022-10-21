// // use glib::subclass;
// // use glib::subclass::prelude::*;
// // use glib::translate::*;
// //
// // use glib::{glib_object_subclass, glib_object_impl, glib_wrapper, glib_object_wrapper, ToValue, StaticType, Cast};
// //
// // use gio::ListStoreExt;
//
// use gesticle::gestures::{GestureType, SwipeDirection, PinchDirection, RotationDirection};
// use gesticle::configuration::GestureActions;
//
// mod imp {
//     use gio::subclass;
//     use std::cell::RefCell;
//
//     use super::*;
//
//     // The actual data structure that stores our values. This is not accessible
//     // directly from the outside.
//     pub struct GestureSettingPrivate {
//         config: RefCell<String>,
//         direction: RefCell<String>,
//         category: RefCell<String>,
//         app: RefCell<Option<String>>,
//         action: RefCell<Option<String>>,
//         inherited: RefCell<Option<String>>,
//         enabled: RefCell<bool>,
//     }
//
//     // GObject property definitions for our two values
//     static PROPERTIES: [subclass::Property; 7] = [
//         subclass::Property("config", |name| {
//             glib::ParamSpec::string(
//                 name,
//                 "Config",
//                 "Config",
//                 None, // Default value
//                 glib::ParamFlags::READWRITE,
//             )
//         }),
//         subclass::Property("direction", |name| {
//             glib::ParamSpec::string(
//                 name,
//                 "Direction",
//                 "Direction",
//                 None, // Default value
//                 glib::ParamFlags::READWRITE,
//             )
//         }),
//         subclass::Property("category", |name| {
//             glib::ParamSpec::string(
//                 name,
//                 "Category",
//                 "Category",
//                 None, // Default value
//                 glib::ParamFlags::READWRITE,
//             )
//         }),
//         subclass::Property("action", |name| {
//             glib::ParamSpec::string(
//                 name,
//                 "Action",
//                 "Action",
//                 None, // Default value
//                 glib::ParamFlags::READWRITE,
//             )
//         }),
//         subclass::Property("app", |name| {
//             glib::ParamSpec::string(
//                 name,
//                 "App",
//                 "App",
//                 None, // Default value
//                 glib::ParamFlags::READWRITE,
//             )
//         }),
//         subclass::Property("inherited", |name| {
//             glib::ParamSpec::string(
//                 name,
//                 "Inherited Action",
//                 "Inherited Action",
//                 None,
//                 glib::ParamFlags::READWRITE,
//             )
//         }),
//         subclass::Property("enabled", |name| {
//             glib::ParamSpec::boolean(
//                 name,
//                 "Enabled",
//                 "Enabled",
//                 false,
//                 glib::ParamFlags::READWRITE,
//             )
//         }),
//     ];
//
//     // Basic declaration of our type for the GObject type system
//     impl ObjectSubclass for GestureSettingPrivate {
//         const NAME: &'static str = "GestureSetting";
//         type ParentType = glib::Object;
//         type Instance = subclass::simple::InstanceStruct<Self>;
//         type Class = subclass::simple::ClassStruct<Self>;
//
//         glib_object_subclass!();
//
//         // Called exactly once before the first instantiation of an instance. This
//         // sets up any type-specific things, in this specific case it installs the
//         // properties so that GObject knows about their existence and they can be
//         // used on instances of our type
//         fn class_init(klass: &mut Self::Class) {
//             klass.install_properties(&PROPERTIES);
//         }
//
//         // Called once at the very beginning of instantiation of each instance and
//         // creates the data structure that contains all our state
//         fn new() -> Self {
//             Self {
//                 config: RefCell::new("".to_string()),
//                 direction: RefCell::new("".to_string()),
//                 category: RefCell::new("".to_string()),
//                 app: RefCell::new(None),
//                 action: RefCell::new(None),
//                 inherited: RefCell::new(None),
//                 enabled: RefCell::new(false),
//             }
//         }
//     }
//
//     // The ObjectImpl trait provides the setters/getters for GObject properties.
//     // Here we need to provide the values that are internally stored back to the
//     // caller, or store whatever new value the caller is providing.
//     //
//     // This maps between the GObject properties and our internal storage of the
//     // corresponding values of the properties.
//     impl ObjectImpl for GestureSettingPrivate {
//         glib_object_impl!();
//
//         fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
//             let prop = &PROPERTIES[id];
//
//             match *prop {
//                 subclass::Property("config", ..) => {
//                     let config = value
//                         .get()
//                         .expect("type conformity checked by `Object::set_property`")
//                         .unwrap();
//                     self.config.replace(config);
//                 }
//                 subclass::Property("direction", ..) => {
//                     let direction = value
//                         .get()
//                         .expect("type conformity checked by `Object::set_property`")
//                         .unwrap();
//                     self.direction.replace(direction);
//                 }
//                 subclass::Property("category", ..) => {
//                     let category = value
//                         .get()
//                         .expect("type conformity checked by `Object::set_property`")
//                         .unwrap();
//                     self.category.replace(category);
//                 }
//                 subclass::Property("app", ..) => {
//                     let action = value
//                         .get()
//                         .expect("type conformity checked by `Object::set_property`");
//                     self.app.replace(action);
//                 }
//                 subclass::Property("action", ..) => {
//                     let action = value
//                         .get()
//                         .expect("type conformity checked by `Object::set_property`");
//                     self.action.replace(action);
//                 }
//                 subclass::Property("enabled", ..) => {
//                     let enabled = value
//                         .get_some()
//                         .expect("type conformity checked by `Object::set_property`");
//                     self.enabled.replace(enabled);
//                 }
//                 subclass::Property("inherited", ..) => {
//                     let inherited = value
//                         .get()
//                         .expect("type conformity checked by `Object::set_property`");
//                     self.inherited.replace(inherited);
//                 }
//                 _ => unimplemented!(),
//             }
//         }
//
//         fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
//             let prop = &PROPERTIES[id];
//
//             match *prop {
//                 subclass::Property("config", ..) => Ok(self.config.borrow().to_value()),
//                 subclass::Property("direction", ..) => Ok(self.direction.borrow().to_value()),
//                 subclass::Property("category", ..) => Ok(self.category.borrow().to_value()),
//                 subclass::Property("app", ..) => Ok(self.app.borrow().to_value()),
//                 subclass::Property("action", ..) => Ok(self.action.borrow().as_ref().or(Some(&"".to_owned())).to_value()),
//                 subclass::Property("inherited", ..) => Ok(self.inherited.borrow().to_value()),
//                 subclass::Property("enabled", ..) => Ok(self.enabled.borrow().to_value()),
//                 _ => unimplemented!(),
//             }
//         }
//     }
// }
//
// // Public part of the GestureSetting type. This behaves like a normal gtk-rs-style GObject
// // binding
// glib_wrapper! {
//         pub struct GestureSetting(Object<subclass::simple::InstanceStruct<imp::GestureSettingPrivate>, subclass::simple::ClassStruct<imp::GestureSettingPrivate>, RowDataClass>);
//
//         match fn {
//             get_type => || imp::GestureSettingPrivate::get_type().to_glib(),
//         }
//     }
//
// // Constructor for new instances. This simply calls glib::Object::new() with
// // initial values for properties and then returns the new instance
// impl GestureSetting {
//     pub fn new_cfg(gesture_type: &GestureType, app: Option<&str>, config: &GestureActions) -> GestureSetting {
//         let setting = &gesture_type.to_config();
//         let inherited = config.get_for_app(setting, None);
//
//         let action = if config.is_specified(setting, app) {
//             config.get_for_app(setting, app)
//         } else {
//             None
//         };
//
//         Self::new(
//             GestureActions::key_for_app(gesture_type.to_config(), app),
//             Self::gesture_direction(gesture_type),
//             Self::gesture_category(gesture_type, app),
//             app,
//             action,
//             inherited,
//             !config.is_disabled(setting, app),
//         )
//     }
//
//     pub fn new(
//         config: String,
//         direction: String,
//         category: String,
//         app: Option<&str>,
//         action: Option<String>,
//         inherited: Option<String>,
//         enabled: bool,
//     ) -> GestureSetting {
//         glib::Object::new(Self::static_type(), &[
//             ("config", &config),
//             ("direction", &direction),
//             ("category", &category),
//             ("app", &app),
//             ("action", &action),
//             ("inherited", &inherited),
//             ("enabled", &enabled)
//         ])
//             .expect("Failed to create row data")
//             .downcast()
//             .expect("Created row data is of wrong type")
//     }
//
//     fn gesture_direction(gesture_type: &GestureType) -> String {
//         match gesture_type {
//             GestureType::Swipe(direction, _) => direction.to_string(),
//             GestureType::Rotation(direction, _) => direction.to_string(),
//             GestureType::Pinch(direction, _) => direction.to_string()
//         }
//     }
//
//     fn gesture_category(gesture_type: &GestureType, app: Option<&str>) -> String {
//         let mut category = match gesture_type {
//             GestureType::Swipe(_, fingers) => format!("{} fingers Swipes", fingers),
//             GestureType::Rotation(_, _) => "Rotations".to_owned(),
//             GestureType::Pinch(_, _) => "Pinches".to_owned()
//         };
//
//         if let Some(context) = app {
//             category.push_str(format!(" in {}", context).as_str());
//         }
//         category
//     }
//
//     pub fn create_app_data(store: &gio::ListStore, app: Option<&str>, config: &GestureActions) {
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Up, 3), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Down, 3), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Left, 3), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Right, 3), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Up, 4), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Down, 4), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Left, 4), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Swipe(SwipeDirection::Right, 4), app, config));
//
//         store.append(&GestureSetting::new_cfg(&GestureType::Pinch(PinchDirection::In, 0.0), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Pinch(PinchDirection::Out, 0.0), app, config));
//
//         store.append(&GestureSetting::new_cfg(&GestureType::Rotation(RotationDirection::Left, 0.0), app, config));
//         store.append(&GestureSetting::new_cfg(&GestureType::Rotation(RotationDirection::Right, 0.0), app, config));
//     }
// }
