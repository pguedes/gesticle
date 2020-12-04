use std::fs::create_dir;
use std::fs::File;
use std::path::{Path, PathBuf};

use simplelog::*;
use config::Source;

// these are the prefixes that are not apps...
const CONFIGURATION_PREFIXES: [&'static str; 4] = ["swipe", "rotation", "pinch", "gesture"];

pub fn init_logging(debug: bool, relative_path: Option<&str>) {
    let user_app_home = home_path(".gesticle").expect("cannot find user home");
    if !user_app_home.exists() {
        create_dir(user_app_home).expect("cannot create app dir in user home");
    }

    let log_path = relative_path.unwrap_or(".gesticle/gesticle.log");
    let log_path = home_path(log_path).expect("cannot create log file path");

    if debug {
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Debug, simplelog::Config::default()).unwrap(),
                WriteLogger::new(LevelFilter::Debug, simplelog::Config::default(),
                                 File::create(log_path).unwrap()),
            ]
        ).unwrap();
    } else {
        WriteLogger::init(LevelFilter::Info, simplelog::Config::default(),
                          File::create(log_path).unwrap()).unwrap();
    }
}

pub fn home_path(relative_path: &str) -> Option<PathBuf> {
    match dirs::home_dir() {
        Some(dir) => Some(dir.join(Path::new(relative_path))),
        None => None
    }
}

fn config_file_path(config_path_override: Option<&str>) -> Result<PathBuf, &str> {
    let path_exists = |p: &PathBuf| p.exists();

    match config_path_override {
        Some(o) => Ok(Path::new(o).to_owned()),
        None => {
            home_path(".gesticle/config.toml").
                filter(path_exists).
                or(Some(Path::new("/etc/gesticle/config.toml").to_owned())).
                filter(path_exists).
                ok_or("nothing in ~/.gesticle/config.toml or /etc/gesticle/config.toml")
        }
    }
}

pub struct GestureActions {
    config: config::Config
}

impl GestureActions {
    pub fn new(config_path_override: Option<&str>) -> GestureActions {
        let config_file_path = config_file_path(config_path_override).
            expect("config file not found");

        info!("creating handler from configuration: {:?}", config_file_path);

        let mut settings = config::Config::new();
        settings.merge(config::File::from(config_file_path)).unwrap();

        GestureActions {
            config: settings
        }
    }

    pub fn new_with_config(config: config::Config) -> GestureActions {
        GestureActions {
            config
        }
    }

    pub fn apps(&self) -> Option<Vec<String>> {
        if let Ok(configs) = self.config.collect() {
            Some(
                configs.iter()
                    .filter_map(|(k, _)| {
                        if !CONFIGURATION_PREFIXES.contains(&k.as_str()) {
                            Some(k.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            )
        } else {
            None
        }
    }

    fn get_no_inheritance(&self, setting: &str, app: Option<&str>) -> Option<String> {
        app.map_or(self.get(setting), |a| self.get(format!("{}.{}", a, setting).as_str()))
    }

    pub fn is_specified(&self, setting: &str, app: Option<&str>) -> bool {
        self.get_no_inheritance(setting, app).is_some()
    }

    pub fn is_disabled(&self, setting: &str, app: Option<&str>) -> bool {
        Some("") == self.get_no_inheritance(setting, app).as_deref()
    }

    pub fn key_for_app(setting: String, app: Option<&str>) -> String {
        match app {
            Some(app) => format!("{}.{}", app, setting),
            None => setting
        }
    }

    pub fn get(&self, setting: &str) -> Option<String> {
        self.config.get_str(setting).ok()
    }

    pub fn get_for_app(&self, setting: &str, app: Option<&str>) -> Option<String> {
        app.map_or(self.get(setting), |a| self.get(format!("{}.{}", a, setting).as_str()))
            .or_else(|| self.get(setting))
            .filter(|v| !v.is_empty())
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.config.get_float(key).ok()
    }
}


#[cfg(test)]
mod tests {
    use configuration::GestureActions;

    #[test]
    fn it_works() {
        let mut config = config::Config::new();
        config.set("swipe.up.3", "ctrl+t").unwrap();
        config.set("firefox.swipe.up.3", "ctrl+y").unwrap();
        config.set("chrome.swipe.up.3", "").unwrap();

        let actions = GestureActions::new_with_config(config);

        assert_eq!(actions.get_for_app("swipe.up.3", None), Some("ctrl+t".to_owned()));
        assert_eq!(actions.get_for_app("swipe.up.3", Some("firefox")), Some("ctrl+y".to_owned()));
        assert_eq!(actions.get_for_app("swipe.up.3", Some("gedit")), Some("ctrl+t".to_owned()));
        assert_eq!(actions.get_for_app("swipe.up.3", Some("chrome")), None);
    }
}