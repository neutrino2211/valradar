use std::env;
use homedir::my_home;

pub fn search_module(module_name: &str) -> Option<String> {
    let current_dir = env::current_dir().unwrap();
    let home_dir = match my_home() {
        Ok(dir) => dir.unwrap(),
        Err(_) => return None,
    };

    let mut module_path = current_dir.join(module_name);

    if module_path.exists() {
        return Some(module_path.to_string_lossy().to_string());
    }

    module_path = home_dir.join(".valradar").join("modules").join(module_name);

    if module_path.exists() {
        return Some(module_path.to_string_lossy().to_string());
    }

    None
}