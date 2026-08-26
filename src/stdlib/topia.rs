use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::object::{HashKey, Object};
use topia::{App as TopiaNativeApp, Node as TopiaNode};

/// Constructs the `topia` standard library module hash.
pub fn make_module() -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("_module".to_string()), Object::String("topia".to_string()));
    map.insert(HashKey::String("App".to_string()), Object::Builtin("std:topia:App".to_string()));
    map.insert(HashKey::String("app".to_string()), Object::Builtin("std:topia:App".to_string()));
    map.insert(HashKey::String("TextInput".to_string()), Object::Builtin("std:topia:TextInput".to_string()));

    map.insert(HashKey::String("text_input".to_string()), Object::Builtin("std:topia:TextInput".to_string()));

    map.insert(HashKey::String("Checkbox".to_string()), Object::Builtin("std:topia:Checkbox".to_string()));

    map.insert(HashKey::String("checkbox".to_string()), Object::Builtin("std:topia:Checkbox".to_string()));
    map.insert(HashKey::String("Text".to_string()), Object::Builtin("std:topia:Text".to_string()));
    map.insert(HashKey::String("text".to_string()), Object::Builtin("std:topia:Text".to_string()));
    map.insert(HashKey::String("Button".to_string()), Object::Builtin("std:topia:Button".to_string()));
    map.insert(HashKey::String("button".to_string()), Object::Builtin("std:topia:Button".to_string()));
    map.insert(HashKey::String("VStack".to_string()), Object::Builtin("std:topia:VStack".to_string()));
    map.insert(HashKey::String("vstack".to_string()), Object::Builtin("std:topia:VStack".to_string()));
    map.insert(HashKey::String("HStack".to_string()), Object::Builtin("std:topia:HStack".to_string()));
    map.insert(HashKey::String("hstack".to_string()), Object::Builtin("std:topia:HStack".to_string()));
    map.insert(HashKey::String("Empty".to_string()), Object::Builtin("std:topia:Empty".to_string()));
    map.insert(HashKey::String("Center".to_string()), Object::Builtin("std:topia:Center".to_string()));
    map.insert(HashKey::String("center".to_string()), Object::Builtin("std:topia:Center".to_string()));
    map.insert(HashKey::String("empty".to_string()), Object::Builtin("std:topia:Empty".to_string()));
    map.insert(HashKey::String("Slider".to_string()), Object::Builtin("std:topia:Slider".to_string()));
    map.insert(HashKey::String("slider".to_string()), Object::Builtin("std:topia:Slider".to_string()));
    map.insert(HashKey::String("ScrollArea".to_string()), Object::Builtin("std:topia:ScrollArea".to_string()));
    map.insert(HashKey::String("scroll_area".to_string()), Object::Builtin("std:topia:ScrollArea".to_string()));
    map.insert(HashKey::String("Graph".to_string()), Object::Builtin("std:topia:Graph".to_string()));
    map.insert(HashKey::String("graph".to_string()), Object::Builtin("std:topia:Graph".to_string()));
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

            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("App".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("App".to_string()));
            map.insert(HashKey::String("title".to_string()), Object::String(title));
            map.insert(HashKey::String("width".to_string()), Object::Float(width));
            map.insert(HashKey::String("height".to_string()), Object::Float(height));
            map.insert(HashKey::String("resizable".to_string()), Object::Boolean(resizable));
            map.insert(HashKey::String("scale".to_string()), Object::Float(1.0));
            map.insert(HashKey::String("run".to_string()), Object::Builtin("std:topia:run".to_string()));
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Text" | "text" => {
            if args.is_empty() || args.len() > 2 {
                return Object::Error(format!("Text expects 1 or 2 arguments (text, styling), got {}", args.len()));
            }
            let content = match &args[0] {
                Object::String(s) => s.clone(),
                other => format!("{}", other),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Text".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Text".to_string()));
            map.insert(HashKey::String("text".to_string()), Object::String(content));
            
            if args.len() > 1 {
                if let Object::Hash(rc) = &args[1] {
                    for (k, v) in rc.borrow().iter() {
                        map.insert(k.clone(), v.clone());
                    }
                } else {
                    return Object::Error(format!("Second argument to Text must be a Hash, got {}", args[1]));
                }
            }
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


        "Center" | "center" => {
            if args.is_empty() {
                return Object::Error("Center expects 1 argument (child), got 0".to_string());
            }
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Center".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Center".to_string()));
            map.insert(HashKey::String("child".to_string()), args[0].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        
        "Separator" | "separator" => {
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("Separator".to_string()));
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "ProgressBar" | "progress_bar" => {
            if args.len() < 1 {
                return Object::Error(format!("ProgressBar expects 1 argument (progress), got {}", args.len()));
            }
            let progress = match &args[0] {
                Object::Integer(i) => *i as f32,
                Object::Float(f) => *f as f32,
                _ => return Object::Error(format!("ProgressBar progress must be numeric, got {}", args[0].type_name())),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("ProgressBar".to_string()));
            map.insert(HashKey::String("progress".to_string()), Object::Float(progress as f64));
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Toggle" | "toggle" => {
            if args.len() < 2 { return Object::Error(format!("Toggle expects 2 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("Toggle".to_string()));
            map.insert(HashKey::String("checked".to_string()), args[0].clone());
            map.insert(HashKey::String("on_change".to_string()), args[1].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "Stepper" | "stepper" => {
            if args.len() < 3 { return Object::Error(format!("Stepper expects 3 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("Stepper".to_string()));
            map.insert(HashKey::String("value".to_string()), args[0].clone());
            map.insert(HashKey::String("step".to_string()), args[1].clone());
            map.insert(HashKey::String("on_change".to_string()), args[2].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "ColorWell" | "color_well" => {
            if args.len() < 2 { return Object::Error(format!("ColorWell expects 2 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("ColorWell".to_string()));
            map.insert(HashKey::String("color".to_string()), args[0].clone());
            map.insert(HashKey::String("on_change".to_string()), args[1].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "ComboBox" | "combo_box" => {
            if args.len() < 3 { return Object::Error(format!("ComboBox expects 3 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("ComboBox".to_string()));
            map.insert(HashKey::String("selected".to_string()), args[0].clone());
            map.insert(HashKey::String("options".to_string()), args[1].clone());
            map.insert(HashKey::String("on_change".to_string()), args[2].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "SegmentedControl" | "segmented_control" => {
            if args.len() < 3 { return Object::Error(format!("SegmentedControl expects 3 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("SegmentedControl".to_string()));
            map.insert(HashKey::String("selected".to_string()), args[0].clone());
            map.insert(HashKey::String("segments".to_string()), args[1].clone());
            map.insert(HashKey::String("on_change".to_string()), args[2].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "GroupBox" | "group_box" => {
            if args.len() < 2 { return Object::Error(format!("GroupBox expects 2 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("GroupBox".to_string()));
            map.insert(HashKey::String("title".to_string()), args[0].clone());
            map.insert(HashKey::String("child".to_string()), args[1].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "DisclosureGroup" | "disclosure_group" => {
            if args.len() < 2 { return Object::Error(format!("DisclosureGroup expects 2 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("DisclosureGroup".to_string()));
            map.insert(HashKey::String("title".to_string()), args[0].clone());
            map.insert(HashKey::String("child".to_string()), args[1].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "TabView" | "tab_view" => {
            if args.len() < 3 { return Object::Error(format!("TabView expects 3 arguments")); }
            let mut map = HashMap::new();
            map.insert(HashKey::String("type".to_string()), Object::String("TabView".to_string()));
            map.insert(HashKey::String("tabs".to_string()), args[0].clone());
            map.insert(HashKey::String("selected".to_string()), args[1].clone());
            map.insert(HashKey::String("on_change".to_string()), args[2].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }
        "Empty" | "empty" => {
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Empty".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Empty".to_string()));
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Slider" | "slider" => {
            if args.len() < 3 {
                return Object::Error(format!("Slider expects at least 3 arguments (value, min, max, optional callback), got {}", args.len()));
            }
            let value = match &args[0] {
                Object::Float(f) => *f,
                Object::Integer(i) => *i as f64,
                _ => return Object::Error(format!("Slider value must be numeric, got {}", args[0].type_name())),
            };
            let min = match &args[1] {
                Object::Float(f) => *f,
                Object::Integer(i) => *i as f64,
                _ => return Object::Error(format!("Slider min must be numeric, got {}", args[1].type_name())),
            };
            let max = match &args[2] {
                Object::Float(f) => *f,
                Object::Integer(i) => *i as f64,
                _ => return Object::Error(format!("Slider max must be numeric, got {}", args[2].type_name())),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Slider".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Slider".to_string()));
            map.insert(HashKey::String("value".to_string()), Object::Float(value));
            map.insert(HashKey::String("min".to_string()), Object::Float(min));
            map.insert(HashKey::String("max".to_string()), Object::Float(max));
            
            if args.len() > 3 {
                map.insert(HashKey::String("on_change".to_string()), args[3].clone());
            }
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "ScrollArea" | "scroll_area" => {
            if args.is_empty() {
                return Object::Error("ScrollArea expects 1 argument (children array), got 0".to_string());
            }
            match &args[0] {
                Object::Array(_) => {}
                _ => return Object::Error(format!("ScrollArea expects Array of children, got {}", args[0].type_name())),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("ScrollArea".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("ScrollArea".to_string()));
            map.insert(HashKey::String("children".to_string()), args[0].clone());
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Graph" | "graph" => {
            if args.len() != 5 {
                return Object::Error(format!("Graph expects 5 arguments (points, min_x, max_x, min_y, max_y), got {}", args.len()));
            }
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Graph".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Graph".to_string()));
            map.insert(HashKey::String("points".to_string()), args[0].clone());
            map.insert(HashKey::String("min_x".to_string()), args[1].clone());
            map.insert(HashKey::String("max_x".to_string()), args[2].clone());
            map.insert(HashKey::String("min_y".to_string()), args[3].clone());
            map.insert(HashKey::String("max_y".to_string()), args[4].clone());
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
            let mut scale = 1.0f32;
            
            if let Object::Hash(rc) = app_obj {
                let map = rc.borrow();
                if let Some(Object::String(s)) = map.get(&HashKey::String("title".to_string())) {
                    title = s.clone();
                }
                if let Some(Object::Float(w)) = map.get(&HashKey::String("width".to_string())) {
                    width = *w as f32;
                }
                if let Some(Object::Float(h)) = map.get(&HashKey::String("height".to_string())) {
                    height = *h as f32;
                }
                if let Some(Object::Boolean(r)) = map.get(&HashKey::String("resizable".to_string())) {
                    resizable = *r;
                }
                if let Some(Object::Float(s)) = map.get(&HashKey::String("scale".to_string())) {
                    scale = *s as f32;
                }
            } else if *app_obj != Object::Null {
                return Object::Error(format!("run expects App object or Null as first argument, got {}", app_obj.type_name()));
            }
            
            let native_app = TopiaNativeApp::new(title, width, height)
                .with_resizable(resizable)
                .with_scale(scale);
            let mut is_verbose = false;
            for arg in std::env::args() {
                if arg == "--verbose" {
                    is_verbose = true;
                }
            }
            
            let res = native_app.run(move || {
                let vb_clone = view_builder_obj.clone();
                match &vb_clone {
                    Object::Function { .. } | Object::Builtin(_) => {
                        let eval_result = crate::evaluator::apply_function(vb_clone.clone(), vec![]);
                        if let Object::Error(err) = &eval_result {
                            eprintln!("[Topia Render Error]: {}", err);
                        }
                        if is_verbose {
                            println!("[Topia Verbose] UI Tree:\n{}", eval_result);
                        }
                        object_to_node(&eval_result)
                    }
                    _ => TopiaNode::Empty,
                }
            });

            match res {
                Ok(_) => Object::Null,
                Err(err) => Object::Error(err),
            }
        }

        "TextInput" | "text_input" | "textinput" => {
            if args.is_empty() {
                return Object::Error("TextInput expects at least 1 argument (text), got 0".to_string());
            }
            let text = match &args[0] {
                Object::String(s) => s.clone(),
                other => format!("{}", other),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("TextInput".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("TextInput".to_string()));
            map.insert(HashKey::String("text".to_string()), Object::String(text));
            if args.len() > 1 {
                map.insert(HashKey::String("on_change".to_string()), args[1].clone());
            }
            Object::Hash(Rc::new(RefCell::new(map)))
        }

        "Checkbox" | "checkbox" => {
            if args.len() < 2 {
                return Object::Error(format!("Checkbox expects at least 2 arguments (checked, label), got {}", args.len()));
            }
            let checked = match &args[0] {
                Object::Boolean(b) => *b,
                _ => return Object::Error(format!("Checkbox 'checked' must be Boolean, got {}", args[0].type_name())),
            };
            let label = match &args[1] {
                Object::String(s) => s.clone(),
                other => format!("{}", other),
            };
            let mut map = HashMap::new();
            map.insert(HashKey::String("_type".to_string()), Object::String("Checkbox".to_string()));
            map.insert(HashKey::String("__type__".to_string()), Object::String("Checkbox".to_string()));
            map.insert(HashKey::String("checked".to_string()), Object::Boolean(checked));
            map.insert(HashKey::String("label".to_string()), Object::String(label));
            if args.len() > 2 {
                map.insert(HashKey::String("on_change".to_string()), args[2].clone());
            }
            Object::Hash(Rc::new(RefCell::new(map)))
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
                    } else if map.contains_key(&HashKey::String("checked".to_string())) {
                        "Checkbox"
                    } else if map.contains_key(&HashKey::String("min".to_string())) && map.contains_key(&HashKey::String("max".to_string())) {
                        "Slider"
                    } else if map.contains_key(&HashKey::String("text".to_string())) && map.contains_key(&HashKey::String("on_change".to_string())) {
                        "TextInput"
                    } else if map.contains_key(&HashKey::String("child".to_string())) {
                        "Center"
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
                    let size = match map.get(&HashKey::String("size".to_string())) {
                        Some(Object::Integer(i)) => Some(*i as f32),
                        Some(Object::Float(f)) => Some(*f as f32),
                        _ => None,
                    };
                    let bold = match map.get(&HashKey::String("bold".to_string())) {
                        Some(Object::Boolean(b)) => *b,
                        _ => false,
                    };
                    TopiaNode::text_styled(text, size, bold)
                }
                "TextInput" | "textinput" | "text_input" => {
                    let text = match map.get(&HashKey::String("text".to_string())) {
                        Some(Object::String(s)) => s.clone(),
                        Some(other) => format!("{}", other),
                        None => String::new(),
                    };
                    if let Some(cb_obj) = map.get(&HashKey::String("on_change".to_string())).cloned() {
                        TopiaNode::text_input(text, move |new_text| {
                            crate::evaluator::apply_function(cb_obj.clone(), vec![Object::String(new_text)]);
                        })
                    } else {
                        TopiaNode::text_input(text, |_| {})
                    }
                }
                "Checkbox" | "checkbox" => {
                    let checked = match map.get(&HashKey::String("checked".to_string())) {
                        Some(Object::Boolean(b)) => *b,
                        _ => false,
                    };
                    let label = match map.get(&HashKey::String("label".to_string())) {
                        Some(Object::String(s)) => s.clone(),
                        Some(other) => format!("{}", other),
                        None => String::new(),
                    };
                    if let Some(cb_obj) = map.get(&HashKey::String("on_change".to_string())).cloned() {
                        TopiaNode::checkbox(checked, label, move |new_val| {
                            let res = crate::evaluator::apply_function(cb_obj.clone(), vec![Object::Boolean(new_val)]);
                            if let Object::Error(err) = res {
                                eprintln!("[Topia Checkbox Callback Error]: {}", err);
                            }
                        })
                    } else {
                        TopiaNode::checkbox(checked, label, |_| {})
                    }
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
                "Slider" | "slider" => {
                    let value = match map.get(&HashKey::String("value".to_string())) {
                        Some(Object::Float(f)) => *f as f32,
                        Some(Object::Integer(i)) => *i as f32,
                        _ => 0.0,
                    };
                    let min = match map.get(&HashKey::String("min".to_string())) {
                        Some(Object::Float(f)) => *f as f32,
                        Some(Object::Integer(i)) => *i as f32,
                        _ => 0.0,
                    };
                    let max = match map.get(&HashKey::String("max".to_string())) {
                        Some(Object::Float(f)) => *f as f32,
                        Some(Object::Integer(i)) => *i as f32,
                        _ => 100.0,
                    };
                    if let Some(cb_obj) = map.get(&HashKey::String("on_change".to_string())).cloned() {
                        TopiaNode::slider(value, min, max, move |new_val| {
                            let res = crate::evaluator::apply_function(cb_obj.clone(), vec![Object::Float(new_val as f64)]);
                            if let Object::Error(err) = res {
                                eprintln!("[Topia Slider Callback Error]: {}", err);
                            }
                        })
                    } else {
                        TopiaNode::slider(value, min, max, |_| {})
                    }
                }
                "ScrollArea" | "scroll_area" => {
                    let children = match map.get(&HashKey::String("children".to_string())) {
                        Some(Object::Array(arr)) => {
                            arr.borrow().iter().map(object_to_node).collect()
                        }
                        _ => vec![],
                    };
                    TopiaNode::scroll_area(children)
                }
                "Graph" | "graph" => {
                    let mut points = vec![];
                    if let Some(Object::Array(arr)) = map.get(&HashKey::String("points".to_string())) {
                        for p in arr.borrow().iter() {
                            if let Object::Array(point_arr) = p {
                                let point = point_arr.borrow();
                                if point.len() == 2 {
                                    let x = match &point[0] {
                                        Object::Float(f) => *f as f32,
                                        Object::Integer(i) => *i as f32,
                                        _ => 0.0,
                                    };
                                    let y = match &point[1] {
                                        Object::Float(f) => *f as f32,
                                        Object::Integer(i) => *i as f32,
                                        _ => 0.0,
                                    };
                                    points.push((x, y));
                                }
                            }
                        }
                    }
                    
                    let get_float = |key: &str| -> f32 {
                        match map.get(&HashKey::String(key.to_string())) {
                            Some(Object::Float(f)) => *f as f32,
                            Some(Object::Integer(i)) => *i as f32,
                            _ => 0.0,
                        }
                    };
                    
                    TopiaNode::graph(
                        points,
                        get_float("min_x"),
                        get_float("max_x"),
                        get_float("min_y"),
                        get_float("max_y")
                    )
                }
                
                "Scale" | "scale" => {
                    let scale_val = match map.get(&HashKey::String("scale".to_string())) {
                        Some(Object::Float(f)) => *f as f32,
                        Some(Object::Integer(i)) => *i as f32,
                        _ => 1.0,
                    };
                    let child_node = match map.get(&HashKey::String("child".to_string())) {
                        Some(child_obj) => object_to_node(child_obj),
                        None => TopiaNode::Empty,
                    };
                    TopiaNode::scale(scale_val, child_node)
                }

                "Center" | "center" => {
                    let child_node = match map.get(&HashKey::String("child".to_string())) {
                        Some(child_obj) => object_to_node(child_obj),
                        None => TopiaNode::Empty,
                    };
                    TopiaNode::center(child_node)
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
