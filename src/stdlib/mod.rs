pub mod math;
pub mod fs;
pub mod json;
pub mod os;
pub mod time;

use std::cell::RefCell;
use std::path::PathBuf;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct FxConfig {
    pub allow_fs: bool,
    pub allow_os: bool,
    pub fs_root: Option<PathBuf>,
    pub max_file_size: usize,
}

impl Default for FxConfig {
    fn default() -> Self {
        FxConfig {
            allow_fs: true,
            allow_os: true,
            fs_root: None,
            max_file_size: 10 * 1024 * 1024,
        }
    }
}

thread_local! {
    static CONFIG: RefCell<FxConfig> = RefCell::new(FxConfig::default());
}

pub fn set_config(config: FxConfig) {
    CONFIG.with(|c| *c.borrow_mut() = config);
}

pub fn get_config() -> FxConfig {
    CONFIG.with(|c| c.borrow().clone())
}

pub fn load_std_module(path: &str) -> Option<Object> {
    let clean_path = path.trim_start_matches("std:").trim_start_matches("std/");
    let config = get_config();
    match clean_path {
        "math" => Some(math::make_module()),
        "fs" => {
            if !config.allow_fs {
                Some(Object::Error("permission denied: filesystem access is disabled".to_string()))
            } else {
                Some(fs::make_module())
            }
        }
        "json" => Some(json::make_module()),
        "os" => {
            if !config.allow_os {
                Some(Object::Error("permission denied: OS access is disabled".to_string()))
            } else {
                Some(os::make_module())
            }
        }
        "time" => Some(time::make_module()),
        _ => None,
    }
}

pub fn apply_std_builtin(full_name: &str, args: Vec<Object>) -> Option<Object> {
    if !full_name.starts_with("std:") {
        return None;
    }
    let parts: Vec<&str> = full_name.splitn(3, ':').collect();
    if parts.len() != 3 {
        return None;
    }
    let module = parts[1];
    let func = parts[2];

    let result = match module {
        "math" => math::apply(func, args),
        "fs" => fs::apply(func, args),
        "json" => json::apply(func, args),
        "os" => os::apply(func, args),
        "time" => time::apply(func, args),
        _ => return None,
    };
    Some(result)
}
