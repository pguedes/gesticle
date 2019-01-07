
use dirs;
use std::path::Path;
use std::path::PathBuf;


pub fn home_path(relative_path: &str) -> Option<PathBuf> {
    match dirs::home_dir() {
        Some(dir) => Some(dir.join(Path::new(relative_path))),
        None => None
    }
}

pub fn config_file_path(config_path_override: Option<&str>) -> Result<PathBuf, &str> {

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