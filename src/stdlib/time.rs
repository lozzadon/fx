use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crate::object::{HashKey, Object};

pub fn make_module() -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("now_ms".to_string()), Object::Builtin("std:time:now_ms".to_string()));
    map.insert(HashKey::String("now_secs".to_string()), Object::Builtin("std:time:now_secs".to_string()));
    map.insert(HashKey::String("sleep_ms".to_string()), Object::Builtin("std:time:sleep_ms".to_string()));
    Object::Hash(Rc::new(RefCell::new(map)))
}

pub fn apply(name: &str, args: Vec<Object>) -> Object {
    match name {
        "now_ms" => {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(dur) => Object::Integer(dur.as_millis() as i64),
                Err(e) => Object::Error(e.to_string()),
            }
        }
        "now_secs" => {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(dur) => Object::Integer(dur.as_secs() as i64),
                Err(e) => Object::Error(e.to_string()),
            }
        }
        "sleep_ms" => {
            if args.len() != 1 {
                return Object::Error(format!("sleep_ms expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::Integer(ms) => {
                    if *ms > 0 {
                        std::thread::sleep(Duration::from_millis(*ms as u64));
                    }
                    Object::Null
                }
                _ => Object::Error("sleep_ms expects integer milliseconds".to_string()),
            }
        }
        _ => Object::Error(format!("unknown std:time function '{}'", name)),
    }
}
