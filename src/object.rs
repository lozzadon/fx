use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::Statement;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    Integer(i64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Object>),
    Hash(HashMap<HashKey, Object>),
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    Iterator {
        target: Box<Object>,
        current: i64,
    },
    ReturnValue(Box<Object>),
    Function {
        parameters: Vec<(String, Option<String>)>,
        return_type: Option<String>,
        body: Statement,
        env: Rc<RefCell<Environment>>,
    },
    Builtin(String),
    Null,
    Error(String),
    Break,
    Continue,
}

impl Object {
    pub fn type_name(&self) -> String {
        match self {
            Object::Integer(_) => "Int".to_string(),
            Object::Float(_) => "Float".to_string(),
            Object::Boolean(_) => "Bool".to_string(),
            Object::String(_) => "String".to_string(),
            Object::Array(_) => "Array".to_string(),
            Object::Hash(_) => "Dict".to_string(),
            Object::Range { .. } => "Range".to_string(),
            Object::Function { .. } => "Func".to_string(),
            Object::Builtin(_) => "Func".to_string(),
            Object::Null => "Void".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

impl Object {
    pub fn get_hash_key(&self) -> Result<HashKey, String> {
        match self {
            Object::Integer(val) => Ok(HashKey::Integer(*val)),
            Object::Boolean(val) => Ok(HashKey::Boolean(*val)),
            Object::String(val) => Ok(HashKey::String(val.clone())),
            _ => Err(format!("unusable as hash key: {}", self)),
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Object::Integer(val) => write!(f, "{}", val),
            Object::Float(val) => write!(f, "{}", val),
            Object::Boolean(val) => write!(f, "{}", val),
            Object::String(val) => write!(f, "{}", val),
            Object::Range { start, end, inclusive } => {
                if *inclusive {
                    write!(f, "{}..={}", start, end)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
            Object::Iterator { target, current } => write!(f, "<iterator @ {} for {}>", current, target),
            Object::Array(elements) => {
                let formatted_elements: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", formatted_elements.join(", "))
            }
            Object::Hash(pairs) => {
                let mut formatted_pairs: Vec<String> = pairs.iter().map(|(k, v)| {
                    let key_str = match k {
                        HashKey::Integer(val) => val.to_string(),
                        HashKey::Boolean(val) => val.to_string(),
                        HashKey::String(val) => format!("\"{}\"", val),
                    };
                    format!("{}: {}", key_str, v)
                }).collect();
                formatted_pairs.sort(); // Sort for consistent output in tests
                write!(f, "{{{}}}", formatted_pairs.join(", "))
            }
            Object::ReturnValue(val) => write!(f, "{}", val),
            Object::Function { parameters, return_type, .. } => {
                let params: Vec<String> = parameters.iter().map(|(name, typ)| {
                    if let Some(t) = typ {
                        format!("{}: {}", name, t)
                    } else {
                        name.clone()
                    }
                }).collect();
                let ret = if let Some(rt) = return_type {
                    format!(" -> {}", rt)
                } else {
                    "".to_string()
                };
                write!(f, "func({}){} {{ ... }}", params.join(", "), ret)
            }
            Object::Builtin(name) => write!(f, "[built-in function {}]", name),
            Object::Null => write!(f, "null"),
            Object::Error(msg) => write!(f, "ERROR: {}", msg),
            Object::Break => write!(f, "BREAK"),
            Object::Continue => write!(f, "CONTINUE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    store: HashMap<String, (Object, bool)>, // (value, is_mutable)
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut store = HashMap::new();
        store.insert("len".to_string(), (Object::Builtin("len".to_string()), false));
        store.insert("push".to_string(), (Object::Builtin("push".to_string()), false));
        store.insert("pop".to_string(), (Object::Builtin("pop".to_string()), false));
        store.insert("print".to_string(), (Object::Builtin("print".to_string()), false));
        store.insert("map".to_string(), (Object::Builtin("map".to_string()), false));
        store.insert("filter".to_string(), (Object::Builtin("filter".to_string()), false));
        store.insert("reduce".to_string(), (Object::Builtin("reduce".to_string()), false));
        store.insert("import".to_string(), (Object::Builtin("import".to_string()), false));
        store.insert("split".to_string(), (Object::Builtin("split".to_string()), false));
        store.insert("trim".to_string(), (Object::Builtin("trim".to_string()), false));
        store.insert("replace".to_string(), (Object::Builtin("replace".to_string()), false));
        store.insert("join".to_string(), (Object::Builtin("join".to_string()), false));
        store.insert("contains".to_string(), (Object::Builtin("contains".to_string()), false));
        store.insert("starts_with".to_string(), (Object::Builtin("starts_with".to_string()), false));
        store.insert("ends_with".to_string(), (Object::Builtin("ends_with".to_string()), false));
        store.insert("to_upper".to_string(), (Object::Builtin("to_upper".to_string()), false));
        store.insert("to_lower".to_string(), (Object::Builtin("to_lower".to_string()), false));
        store.insert("substring".to_string(), (Object::Builtin("substring".to_string()), false));
        
        Environment {
            store,
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some((val, _)) => Some(val.clone()),
            None => match &self.outer {
                Some(outer_env) => outer_env.borrow().get(name),
                None => None,
            },
        }
    }
    
    #[allow(dead_code)]
    pub fn is_mutable(&self, name: &str) -> Option<bool> {
        match self.store.get(name) {
            Some((_, mutable)) => Some(*mutable),
            None => match &self.outer {
                Some(outer_env) => outer_env.borrow().is_mutable(name),
                None => None,
            }
        }
    }

    pub fn set(&mut self, name: String, val: Object, is_mutable: bool) -> Object {
        self.store.insert(name, (val.clone(), is_mutable));
        val
    }
    
    pub fn assign(&mut self, name: &str, value: Object) -> Result<(), String> {
        if let Some((_, is_mutable)) = self.store.get(name) {
            if *is_mutable {
                self.store.insert(name.to_string(), (value, true));
                return Ok(());
            } else {
                return Err(format!("cannot assign to immutable variable '{}'", name));
            }
        }

        match &self.outer {
            Some(outer) => outer.borrow_mut().assign(name, value),
            None => Err(format!("variable '{}' not found", name)),
        }
    }

    pub fn get_all(&self) -> HashMap<String, (Object, bool)> {
        self.store.clone()
    }
}
