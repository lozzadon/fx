use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::object::{HashKey, Object};
use topia::{App as TopiaNativeApp, Node as TopiaNode};

thread_local! {
    static CURRENT_APP: RefCell<Option<TopiaNativeApp>> = RefCell::new(None);
}

/// Constructs the `topia` standard library module hash.
pub fn make_module() -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("_module".to_string()), Object::String("topia".to_string()));
    map.insert(HashKey::String("App".to_string()), Object::Builtin("std:topia:App".to_string()));
    map.insert(HashKey::String("app".to_string()), Object::Builtin("std:topia:App".to_string()));
    map.insert(HashKey::String("Text".to_string()), Object::Builtin("std:topia:Text".to_string()));
    map.insert(HashKey::String("text".to_string()), Object::Builtin("std:topia:Text".to_string()));
    map.insert(HashKey::String("Button".to_string()), Object::Builtin("std:topia:Button".to_string()));
    map.insert(HashKey::String("button".to_string()), Object::Builtin("std:topia:Button".to_string()));
    map.insert(HashKey::String("VStack".to_string()), Object::Builtin("std:topia:VStack".to_string()));
    map.insert(HashKey::String("vstack".to_string()), Object::Builtin("std:topia:VStack".to_string()));
    map.insert(HashKey::String("HStack".to_string()), Object::Builtin("std:topia:HStack".to_string()));
    map.insert(HashKey::String("hstack".to_string()), Object::Builtin("std:topia:HStack".to_string()));
    map.insert(HashKey::String("Empty".to_string()), Object::Builtin("std:topia:Empty".to_string()));
    map.insert(HashKey::String("empty".to_string()), Object::Builtin("std:topia:Empty".to_string()));
    map.insert(HashKey::String("run".to_string()), Object::Builtin("std:topia:run".to_string()));
    Object::Hash(Rc::new(RefCell::new(map)))
}

/// Applies a `std:topia:*` builtin function.
pub fn apply(name: &str, args: Vec<Object>) -> Object {
    match name {
        "App" | "app" => {
            if args.len() < 3 {
                return Object::Error(format!("App expects 3 arguments (title, width, height), got {}", args.len()));
            }
            let title = match &args[0] {
                Object::String(s) => s.clone(),
                _ => return Object::Error(format!("App title must be String, got {}", args[0].type_name())),
            };
            let width = match &args[1] {
                Object::Integer(i) => *i as f64,
                Object::Float(f) => *f,
                _ => return Object::Error(format!("App width must be numeric, got {}", args[1].type_name())),
            };
            let height = match &args[2] {
                Object::Integer(i) => *i as f64,
                Object::Float(f) => *f,
                _ => return Object::Error(format!("App height must be numeric, got {}", args[2].type_name())),
            };
            let resizable = if args.len() > 3 {
                match &args[3] {
                    Object::Boolean(b) => *b,
                    _ => true,
                }
            } else {
                true
            };

            let app_instance = TopiaNativeApp::new(&title, width as f32, height as f32).with_resizable(resizable);
            CURRENT_APP.with(|c| *c.borrow_mut() = Some(app_instance));

            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("App".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("App".to_string()));
            map.insert(HashKey::String("title".to_string()), Object::String(title));
            map.insert(HashKey::String("width".to_string()), Object::Float(width));
            map.insert(HashKey::String("height".to_string()), Object::Float(height));
            map.insert(HashKey::String("resizable".to_string()), Object::Boolean(resizable));
            map.insert(HashKey::String("run".to_string()), Object::Builtin("std:topia:run".to_string()));
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Text" | "text" => {
            if args.len() != 1 {
                return Object::Error(format!("Text expects 1 argument (text), got {}", args.len()));
            }
            let content = match &args[0] {
                Object::String(s) => s.clone(),
                other => format!("{}", other),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Text".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Text".to_string()));
            map.insert(HashKey::String("text".to_string()), Object::String(content));
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Button" | "button" => {
            if args.len() != 2 {
                return Object::Error(format!("Button expects 2 arguments (label, callback), got {}", args.len()));
            }
            let label = match &args[0] {
                Object::String(s) => s.clone(),
                _ => return Object::Error(format!("Button label must be String, got {}", args[0].type_name())),
            };
            match &args[1] {
                Object::Function { .. } | Object::Builtin(_) => {}
                _ => return Object::Error(format!("Button callback must be a function, got {}", args[1].type_name())),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Button".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Button".to_string()));
            map.insert(HashKey::String("label".to_string()), Object::String(label));
            map.insert(HashKey::String("on_click".to_string()), args[1].clone());
            map.insert(HashKey::String("callback".to_string()), args[1].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "VStack" | "vstack" => {
            if args.is_empty() {
                return Object::Error("VStack expects 1 argument (children array), got 0".to_string());
            }
            match &args[0] {
                Object::Array(_) => {}
                _ => return Object::Error(format!("VStack expects Array of children, got {}", args[0].type_name())),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("VStack".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("VStack".to_string()));
            map.insert(HashKey::String("children".to_string()), args[0].clone());
            if args.len() > 1 {
                if let Object::Float(s) = &args[1] {
                    map.insert(HashKey::String("spacing".to_string()), Object::Float(*s));
                } else if let Object::Integer(s) = &args[1] {
                    map.insert(HashKey::String("spacing".to_string()), Object::Float(*s as f64));
                }
            }
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "HStack" | "hstack" => {
            if args.is_empty() {
                return Object::Error("HStack expects 1 argument (children array), got 0".to_string());
            }
            match &args[0] {
                Object::Array(_) => {}
                _ => return Object::Error(format!("HStack expects Array of children, got {}", args[0].type_name())),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("HStack".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("HStack".to_string()));
            map.insert(HashKey::String("children".to_string()), args[0].clone());
            if args.len() > 1 {
                if let Object::Float(s) = &args[1] {
                    map.insert(HashKey::String("spacing".to_string()), Object::Float(*s));
                } else if let Object::Integer(s) = &args[1] {
                    map.insert(HashKey::String("spacing".to_string()), Object::Float(*s as f64));
                }
            }
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Empty" | "empty" => {
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Empty".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Empty".to_string()));
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "run" => {
            if args.is_empty() || args.len() > 2 {
                return Object::Error(format!("run expects 1 or 2 arguments (app, view_builder), got {}", args.len()));
            }
            let (app_obj, view_builder_obj) = if args.len() == 2 {
                (&args[0], args[1].clone())
            } else {
                (&Object::Null, args[0].clone())
            };

            let mut title = "Topia App".to_string();
            let mut width = 800.0f32;
            let mut height = 600.0f32;
            let mut resizable = true;

            if let Object::Hash(rc) = app_obj {
                let map = rc.borrow();
                if let Some(Object::String(t)) = map.get(&HashKey::String("title".to_string())) {
                    title = t.clone();
                }
                if let Some(Object::Float(w)) = map.get(&HashKey::String("width".to_string())) {
                    width = *w as f32;
                } else if let Some(Object::Integer(w)) = map.get(&HashKey::String("width".to_string())) {
                    width = *w as f32;
                }
                if let Some(Object::Float(h)) = map.get(&HashKey::String("height".to_string())) {
                    height = *h as f32;
                } else if let Some(Object::Integer(h)) = map.get(&HashKey::String("height".to_string())) {
                    height = *h as f32;
                }
                if let Some(Object::Boolean(r)) = map.get(&HashKey::String("resizable".to_string())) {
                    resizable = *r;
                }
            } else {
                CURRENT_APP.with(|c| {
                    if let Some(app) = &*c.borrow() {
                        title = app.title.clone();
                        width = app.width;
                        height = app.height;
                        resizable = app.resizable;
                    }
                });
            }

            let native_app = TopiaNativeApp::new(title, width, height).with_resizable(resizable);
            let vb_clone = view_builder_obj;
            let res = native_app.run(move || {
                match &vb_clone {
                    Object::Function { .. } | Object::Builtin(_) => {
                        let eval_result = crate::evaluator::apply_function(vb_clone.clone(), vec![]);
                        object_to_node(&eval_result)
                    }
                    static_obj => object_to_node(static_obj),
                }
            });

            match res {
                Ok(_) => Object::Null,
                Err(err) => Object::Error(err),
            }
        }

        _ => Object::Error(format!("unknown std:topia function '{}'", name)),
    }
}

/// Translates an `f(x)` `Object` tree into a declarative `topia::Node` tree.
pub fn object_to_node(obj: &Object) -> TopiaNode {
    match obj {
        Object::Hash(rc) => {
            let map = rc.borrow();
            let node_type = match map.get(&HashKey::String("_type".to_string()))
                .or_else(|| map.get(&HashKey::String("__type__".to_string())))
                .or_else(|| map.get(&HashKey::String("type".to_string()))) {
                Some(Object::String(s)) => s.as_str(),
                _ => {
                    if map.contains_key(&HashKey::String("children".to_string())) {
                        "VStack"
                    } else if map.contains_key(&HashKey::String("on_click".to_string())) || map.contains_key(&HashKey::String("callback".to_string())) {
                        "Button"
                    } else if map.contains_key(&HashKey::String("text".to_string())) {
                        "Text"
                    } else {
                        return TopiaNode::Empty;
                    }
                }
            };

            match node_type {
                "Text" | "text" => {
                    let text = match map.get(&HashKey::String("text".to_string()))
                        .or_else(|| map.get(&HashKey::String("content".to_string()))) {
                        Some(Object::String(s)) => s.clone(),
                        Some(other) => format!("{}", other),
                        None => String::new(),
                    };
                    TopiaNode::text(text)
                }
                "Button" | "button" => {
                    let label = match map.get(&HashKey::String("label".to_string()))
                        .or_else(|| map.get(&HashKey::String("text".to_string()))) {
                        Some(Object::String(s)) => s.clone(),
                        Some(other) => format!("{}", other),
                        None => "Button".to_string(),
                    };
                    let cb_obj = map.get(&HashKey::String("on_click".to_string()))
                        .or_else(|| map.get(&HashKey::String("callback".to_string())))
                        .cloned()
                        .unwrap_or(Object::Null);
                    TopiaNode::button(label, move || {
                        if cb_obj != Object::Null {
                            let res = crate::evaluator::apply_function(cb_obj.clone(), vec![]);
                            if let Object::Error(err) = res {
                                eprintln!("[Topia Button Callback Error]: {}", err);
                            }
                        }
                    })
                }
                "VStack" | "vstack" => {
                    let children = match map.get(&HashKey::String("children".to_string())) {
                        Some(Object::Array(arr)) => {
                            arr.borrow().iter().map(object_to_node).collect()
                        }
                        _ => vec![],
                    };
                    let spacing = match map.get(&HashKey::String("spacing".to_string())) {
                        Some(Object::Float(f)) => Some(*f as f32),
                        Some(Object::Integer(i)) => Some(*i as f32),
                        _ => None,
                    };
                    if let Some(s) = spacing {
                        TopiaNode::vstack_with_spacing(children, s)
                    } else {
                        TopiaNode::vstack(children)
                    }
                }
                "HStack" | "hstack" => {
                    let children = match map.get(&HashKey::String("children".to_string())) {
                        Some(Object::Array(arr)) => {
                            arr.borrow().iter().map(object_to_node).collect()
                        }
                        _ => vec![],
                    };
                    let spacing = match map.get(&HashKey::String("spacing".to_string())) {
                        Some(Object::Float(f)) => Some(*f as f32),
                        Some(Object::Integer(i)) => Some(*i as f32),
                        _ => None,
                    };
                    if let Some(s) = spacing {
                        TopiaNode::hstack_with_spacing(children, s)
                    } else {
                        TopiaNode::hstack(children)
                    }
                }
                "Empty" | "empty" => TopiaNode::Empty,
                _ => TopiaNode::Empty,
            }
        }
        Object::Array(rc) => {
            let children = rc.borrow().iter().map(object_to_node).collect();
            TopiaNode::vstack(children)
        }
        Object::String(s) => TopiaNode::text(s.clone()),
        Object::Integer(i) => TopiaNode::text(i.to_string()),
        Object::Float(f) => TopiaNode::text(f.to_string()),
        Object::Boolean(b) => TopiaNode::text(b.to_string()),
        Object::Null => TopiaNode::Empty,
        _ => TopiaNode::Empty,
    }
}

/// Alias for `object_to_node`.
pub fn convert_object_to_node(obj: &Object) -> Result<TopiaNode, String> {
    Ok(object_to_node(obj))
}
