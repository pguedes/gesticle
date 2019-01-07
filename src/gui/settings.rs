use gtk;
use gtk::{GridExt, WidgetExt, LabelExt};
use config;

use paths::config_file_path;
use std::collections::HashMap;
use config::Value;

pub struct Settings {
    grid: gtk::Grid,
    config: config::Config,
}

impl Settings {
    pub fn new(grid: gtk::Grid) -> Result<Settings, &'static str> {
        match config_file_path(None) {
            Ok(path) => {
                let mut config = config::Config::new();
                config.merge(config::File::from(path)).unwrap();

                Ok(Settings { config, grid })
            }
            Err(e) => Err(e)
        }
    }

    fn settings(&self) -> HashMap<String, String> {

        let map: HashMap<String, String> = HashMap::new();

        let root = self.config.cache.clone();

        Self::setting("".to_owned(), root, map)
    }

    fn setting(prefix: String, value: Value, mut map: HashMap<String, String>) -> HashMap<String, String> {

        if let Ok(inner) = value.clone().into_table() {
            for (k, v) in inner {
                let path = if prefix.is_empty() {k} else {format!("{}.{}", prefix, k)} ;
                map = Self::setting(path, v, map);
            }
        } else {
            map.insert(prefix, value.into_str().unwrap().to_owned());
        }
        map
    }

    pub fn build_gui(&self) {

        let settings = self.settings();

        let mut row = 1;

        for (setting, value) in &settings {

            row += 1;

            let setting_label = gtk::Label::new(setting.as_str());
            setting_label.set_markup(&format!("<b>{}</b>", setting));
            setting_label.set_halign(gtk::Align::End);
            setting_label.set_margin_top(5);
            setting_label.set_margin_end(5);
            self.grid.attach(&setting_label, 0, row, 1, 1);

            let value_label = gtk::Label::new(value.as_str());
            value_label.set_halign(gtk::Align::Start);
            value_label.set_margin_top(5);
            value_label.set_margin_start(5);
            self.grid.attach(&value_label, 1, row, 1, 1);
        }
    }
}