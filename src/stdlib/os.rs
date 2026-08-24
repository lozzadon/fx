use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::cell::RefCell;
use crate::object::{HashKey, Object};

thread_local! {
    static VIRTUAL_ENV: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

pub fn make_module() -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("args".to_string()), Object::Builtin("std:os:args".to_string()));
    map.insert(HashKey::String("env".to_string()), Object::Builtin("std:os:env".to_string()));
    map.insert(HashKey::String("get_env".to_string()), Object::Builtin("std:os:get_env".to_string()));
    map.insert(HashKey::String("set_env".to_string()), Object::Builtin("std:os:set_env".to_string()));
    map.insert(HashKey::String("exit".to_string()), Object::Builtin("std:os:exit".to_string()));
    map.insert(HashKey::String("platform".to_string()), Object::Builtin("std:os:platform".to_string()));
    map.insert(HashKey::String("getpid".to_string()), Object::Builtin("std:os:getpid".to_string()));
    Object::Hash(Rc::new(RefCell::new(map)))
}

pub fn apply(name: &str, args: Vec<Object>) -> Object {
    let config = crate::stdlib::get_config();
    if !config.allow_os {
        return Object::Error("permission denied: OS access is disabled".to_string());
    }
    match name {
        "args" => {
            let cli_args: Vec<Object> = env::args().map(Object::String).collect();
            Object::Array(Rc::new(RefCell::new(cli_args)))
        }
        "env" => {
            let mut map = HashMap::new();
            for (k, v) in env::vars() {
                map.insert(HashKey::String(k), Object::String(v));
            }
            VIRTUAL_ENV.with(|venv| {
                for (k, v) in venv.borrow().iter() {
                    map.insert(HashKey::String(k.clone()), Object::String(v.clone()));
                }
            });
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "get_env" => {
            if args.len() != 1 {
                return Object::Error(format!("get_env expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(key) => {
                    let mut found = None;
                    VIRTUAL_ENV.with(|venv| {
                        if let Some(val) = venv.borrow().get(key) {
                            found = Some(val.clone());
                        }
                    });
                    if let Some(val) = found {
                        return Object::String(val);
                    }
                    match env::var(key) {
                        Ok(val) => Object::String(val),
                        Err(_) => Object::Null,
                    }
                }
                _ => Object::Error("get_env expects string key".to_string()),
            }
        }
        "set_env" => {
            if args.len() != 2 {
                return Object::Error(format!("set_env expects 2 arguments, got {}", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(k), Object::String(v)) => {
                    VIRTUAL_ENV.with(|venv| {
                        venv.borrow_mut().insert(k.clone(), v.clone());
                    });
                    Object::Boolean(true)
                }
                _ => Object::Error("set_env expects (string, string)".to_string()),
            }
        }
        "exit" => {
            let code = if args.is_empty() {
                0
            } else if let Object::Integer(c) = &args[0] {
                *c as i32
            } else {
                0
            };
            std::process::exit(code);
        }
        "platform" => {
            Object::String(std::env::consts::OS.to_string())
        }
        "getpid" => {
            Object::Integer(std::process::id() as i64)
        }
        _ => Object::Error(format!("unknown std:os function '{}'", name)),
    }
}
