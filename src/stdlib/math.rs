use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::object::{HashKey, Object};

pub fn make_module() -> Object {
    let mut map = HashMap::new();
    
    // Constants
    map.insert(HashKey::String("PI".to_string()), Object::Float(std::f64::consts::PI));
    map.insert(HashKey::String("E".to_string()), Object::Float(std::f64::consts::E));
    
    // Functions
    map.insert(HashKey::String("abs".to_string()), Object::Builtin("std:math:abs".to_string()));
    map.insert(HashKey::String("sqrt".to_string()), Object::Builtin("std:math:sqrt".to_string()));
    map.insert(HashKey::String("pow".to_string()), Object::Builtin("std:math:pow".to_string()));
    map.insert(HashKey::String("floor".to_string()), Object::Builtin("std:math:floor".to_string()));
    map.insert(HashKey::String("ceil".to_string()), Object::Builtin("std:math:ceil".to_string()));
    map.insert(HashKey::String("round".to_string()), Object::Builtin("std:math:round".to_string()));
    map.insert(HashKey::String("sin".to_string()), Object::Builtin("std:math:sin".to_string()));
    map.insert(HashKey::String("cos".to_string()), Object::Builtin("std:math:cos".to_string()));
    map.insert(HashKey::String("tan".to_string()), Object::Builtin("std:math:tan".to_string()));
    map.insert(HashKey::String("log".to_string()), Object::Builtin("std:math:log".to_string()));
    map.insert(HashKey::String("min".to_string()), Object::Builtin("std:math:min".to_string()));
    map.insert(HashKey::String("max".to_string()), Object::Builtin("std:math:max".to_string()));
    
    Object::Hash(Rc::new(RefCell::new(map)))
}

fn to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(i) => Some(*i as f64),
        Object::Float(f) => Some(*f),
        _ => None,
    }
}

pub fn apply(name: &str, args: Vec<Object>) -> Object {
    match name {
        "abs" => {
            if args.len() != 1 {
                return Object::Error(format!("abs expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::Integer(i) => Object::Integer(i.abs()),
                Object::Float(f) => Object::Float(f.abs()),
                _ => Object::Error(format!("abs expects numeric argument, got {}", args[0].type_name())),
            }
        }
        "sqrt" => {
            if args.len() != 1 {
                return Object::Error(format!("sqrt expects 1 argument, got {}", args.len()));
            }
            if let Some(val) = to_f64(&args[0]) {
                if val < 0.0 {
                    return Object::Error("sqrt cannot be called with negative number".to_string());
                }
                Object::Float(val.sqrt())
            } else {
                Object::Error(format!("sqrt expects numeric argument, got {}", args[0].type_name()))
            }
        }
        "pow" => {
            if args.len() != 2 {
                return Object::Error(format!("pow expects 2 arguments, got {}", args.len()));
            }
            match (to_f64(&args[0]), to_f64(&args[1])) {
                (Some(base), Some(exp)) => {
                    if let (Object::Integer(b), Object::Integer(e)) = (&args[0], &args[1]) {
                        if *e >= 0 && *e <= u32::MAX as i64 {
                            if let Some(res) = b.checked_pow(*e as u32) {
                                return Object::Integer(res);
                            }
                        }
                    }
                    let res = base.powf(exp);
                    if res.is_nan() {
                        return Object::Error("pow result is not a number (invalid base/exponent combination)".to_string());
                    }
                    Object::Float(res)
                }
                _ => Object::Error("pow expects numeric arguments".to_string()),
            }
        }
        "floor" => {
            if args.len() != 1 {
                return Object::Error(format!("floor expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::Integer(i) => Object::Integer(*i),
                Object::Float(f) => Object::Float(f.floor()),
                _ => Object::Error(format!("floor expects numeric argument, got {}", args[0].type_name())),
            }
        }
        "ceil" => {
            if args.len() != 1 {
                return Object::Error(format!("ceil expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::Integer(i) => Object::Integer(*i),
                Object::Float(f) => Object::Float(f.ceil()),
                _ => Object::Error(format!("ceil expects numeric argument, got {}", args[0].type_name())),
            }
        }
        "round" => {
            if args.len() != 1 {
                return Object::Error(format!("round expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::Integer(i) => Object::Integer(*i),
                Object::Float(f) => Object::Float(f.round()),
                _ => Object::Error(format!("round expects numeric argument, got {}", args[0].type_name())),
            }
        }
        "sin" => {
            if args.len() != 1 {
                return Object::Error(format!("sin expects 1 argument, got {}", args.len()));
            }
            if let Some(val) = to_f64(&args[0]) {
                Object::Float(val.sin())
            } else {
                Object::Error(format!("sin expects numeric argument, got {}", args[0].type_name()))
            }
        }
        "cos" => {
            if args.len() != 1 {
                return Object::Error(format!("cos expects 1 argument, got {}", args.len()));
            }
            if let Some(val) = to_f64(&args[0]) {
                Object::Float(val.cos())
            } else {
                Object::Error(format!("cos expects numeric argument, got {}", args[0].type_name()))
            }
        }
        "tan" => {
            if args.len() != 1 {
                return Object::Error(format!("tan expects 1 argument, got {}", args.len()));
            }
            if let Some(val) = to_f64(&args[0]) {
                Object::Float(val.tan())
            } else {
                Object::Error(format!("tan expects numeric argument, got {}", args[0].type_name()))
            }
        }
        "log" => {
            if args.len() != 1 {
                return Object::Error(format!("log expects 1 argument, got {}", args.len()));
            }
            if let Some(val) = to_f64(&args[0]) {
                if val <= 0.0 {
                    return Object::Error("log cannot be called with non-positive number".to_string());
                }
                Object::Float(val.ln())
            } else {
                Object::Error(format!("log expects numeric argument, got {}", args[0].type_name()))
            }
        }
        "min" => {
            if args.len() != 2 {
                return Object::Error(format!("min expects 2 arguments, got {}", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::Integer(a), Object::Integer(b)) => Object::Integer(*a.min(b)),
                (Object::Float(a), Object::Float(b)) => Object::Float(a.min(*b)),
                (Object::Integer(a), Object::Float(b)) => Object::Float((*a as f64).min(*b)),
                (Object::Float(a), Object::Integer(b)) => Object::Float(a.min(*b as f64)),
                _ => Object::Error("min expects numeric arguments".to_string()),
            }
        }
        "max" => {
            if args.len() != 2 {
                return Object::Error(format!("max expects 2 arguments, got {}", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::Integer(a), Object::Integer(b)) => Object::Integer(*a.max(b)),
                (Object::Float(a), Object::Float(b)) => Object::Float(a.max(*b)),
                (Object::Integer(a), Object::Float(b)) => Object::Float((*a as f64).max(*b)),
                (Object::Float(a), Object::Integer(b)) => Object::Float(a.max(*b as f64)),
                _ => Object::Error("max expects numeric arguments".to_string()),
            }
        }
        _ => Object::Error(format!("unknown std:math function '{}'", name)),
    }
}
