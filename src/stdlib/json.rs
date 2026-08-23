use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use crate::object::{HashKey, Object};

thread_local! {
    static VISITED_JSON_POINTERS: RefCell<HashSet<*const ()>> = RefCell::new(HashSet::new());
}

pub fn make_module() -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("parse".to_string()), Object::Builtin("std:json:parse".to_string()));
    map.insert(HashKey::String("stringify".to_string()), Object::Builtin("std:json:stringify".to_string()));
    Object::Hash(Rc::new(RefCell::new(map)))
}

pub fn apply(name: &str, args: Vec<Object>) -> Object {
    match name {
        "parse" => {
            if args.len() != 1 {
                return Object::Error(format!("json.parse expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(s) => match parse_json(s) {
                    Ok(obj) => obj,
                    Err(e) => Object::Error(format!("JSON parse error: {}", e)),
                },
                _ => Object::Error(format!("json.parse expects string, got {}", args[0].type_name())),
            }
        }
        "stringify" => {
            if args.len() != 1 {
                return Object::Error(format!("json.stringify expects 1 argument, got {}", args.len()));
            }
            Object::String(stringify_json(&args[0]))
        }
        _ => Object::Error(format!("unknown std:json function '{}'", name)),
    }
}

// -----------------------------------------------------------------------------
// JSON Stringify
// -----------------------------------------------------------------------------

struct JsonPointerGuard(*const ());
impl Drop for JsonPointerGuard {
    fn drop(&mut self) {
        VISITED_JSON_POINTERS.with(|v| v.borrow_mut().remove(&self.0));
    }
}

pub fn stringify_json(obj: &Object) -> String {
    match obj {
        Object::Null => "null".to_string(),
        Object::Boolean(b) => if *b { "true".to_string() } else { "false".to_string() },
        Object::Integer(i) => i.to_string(),
        Object::Float(f) => f.to_string(),
        Object::String(s) => {
            let mut out = String::from("\"");
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '\x08' => out.push_str("\\b"),
                    '\x0C' => out.push_str("\\f"),
                    _ => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Object::Array(rc) => {
            let ptr = rc.as_ptr() as *const ();
            let already_visited = VISITED_JSON_POINTERS.with(|v| !v.borrow_mut().insert(ptr));
            if already_visited {
                return "null".to_string();
            }
            let _guard = JsonPointerGuard(ptr);
            let vec = rc.borrow();
            let elements: Vec<String> = vec.iter().map(stringify_json).collect();
            format!("[{}]", elements.join(","))
        }
        Object::Hash(rc) => {
            let ptr = rc.as_ptr() as *const ();
            let already_visited = VISITED_JSON_POINTERS.with(|v| !v.borrow_mut().insert(ptr));
            if already_visited {
                return "null".to_string();
            }
            let _guard = JsonPointerGuard(ptr);
            let map = rc.borrow();
            let mut entries = Vec::new();
            for (k, v) in map.iter() {
                let key_str = match k {
                    HashKey::String(s) => s.clone(),
                    HashKey::Integer(i) => i.to_string(),
                    HashKey::Boolean(b) => b.to_string(),
                };
                entries.push(format!("\"{}\":{}", key_str, stringify_json(v)));
            }
            entries.sort();
            format!("{{{}}}", entries.join(","))
        }
        Object::StructInstance { fields, .. } => {
            let ptr = fields.as_ptr() as *const ();
            let already_visited = VISITED_JSON_POINTERS.with(|v| !v.borrow_mut().insert(ptr));
            if already_visited {
                return "null".to_string();
            }
            let _guard = JsonPointerGuard(ptr);
            let map = fields.borrow();
            let mut entries = Vec::new();
            for (k, v) in map.iter() {
                entries.push(format!("\"{}\":{}", k, stringify_json(v)));
            }
            entries.sort();
            format!("{{{}}}", entries.join(","))
        }
        _ => "null".to_string(),
    }
}

// -----------------------------------------------------------------------------
// JSON Parser
// -----------------------------------------------------------------------------

struct JsonParser<'a> {
    chars: Vec<char>,
    pos: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        JsonParser {
            chars: input.chars().collect(),
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                ' ' | '\t' | '\n' | '\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn parse_value(&mut self) -> Result<Object, String> {
        self.skip_whitespace();
        let c = match self.peek() {
            Some(ch) => ch,
            None => return Err("Unexpected end of JSON input".to_string()),
        };

        match c {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' => self.parse_string().map(Object::String),
            't' | 'f' => self.parse_bool(),
            'n' => self.parse_null(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character '{}' at position {}", c, self.pos)),
        }
    }

    fn parse_object(&mut self) -> Result<Object, String> {
        self.next_char(); // consume '{'
        self.skip_whitespace();
        let mut map = HashMap::new();

        if let Some('}') = self.peek() {
            self.next_char();
            return Ok(Object::Hash(Rc::new(RefCell::new(map))));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();

            match self.next_char() {
                Some(':') => {}
                Some(c) => return Err(format!("Expected ':' after key, got '{}'", c)),
                None => return Err("Unexpected EOF expecting ':'".to_string()),
            }

            let val = self.parse_value()?;
            map.insert(HashKey::String(key), val);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.next_char();
                }
                Some('}') => {
                    self.next_char();
                    break;
                }
                Some(c) => return Err(format!("Expected ',' or '}}' in object, got '{}'", c)),
                None => return Err("Unexpected EOF in object".to_string()),
            }
        }

        Ok(Object::Hash(Rc::new(RefCell::new(map))))
    }

    fn parse_array(&mut self) -> Result<Object, String> {
        self.next_char(); // consume '['
        self.skip_whitespace();
        let mut vec = Vec::new();

        if let Some(']') = self.peek() {
            self.next_char();
            return Ok(Object::Array(Rc::new(RefCell::new(vec))));
        }

        loop {
            let val = self.parse_value()?;
            vec.push(val);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.next_char();
                }
                Some(']') => {
                    self.next_char();
                    break;
                }
                Some(c) => return Err(format!("Expected ',' or ']' in array, got '{}'", c)),
                None => return Err("Unexpected EOF in array".to_string()),
            }
        }

        Ok(Object::Array(Rc::new(RefCell::new(vec))))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.skip_whitespace();
        if self.next_char() != Some('"') {
            return Err("Expected '\"' at start of string".to_string());
        }

        let mut s = String::new();
        while let Some(c) = self.next_char() {
            if c == '"' {
                return Ok(s);
            } else if c == '\\' {
                match self.next_char() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('/') => s.push('/'),
                    Some('b') => s.push('\x08'),
                    Some('f') => s.push('\x0C'),
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('u') => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            if let Some(h) = self.next_char() {
                                hex.push(h);
                            } else {
                                return Err("Unexpected EOF in unicode escape".to_string());
                            }
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if (0xD800..=0xDBFF).contains(&code) {
                                // High surrogate. Look for following \uXXXX low surrogate
                                if self.chars[self.pos..].starts_with(&['\\', 'u']) && self.chars.len() >= self.pos + 6 {
                                    let mut low_hex = String::new();
                                    let mut is_valid_low_hex = true;
                                    for i in 0..4 {
                                        let ch = self.chars[self.pos + 2 + i];
                                        if ch.is_ascii_hexdigit() {
                                            low_hex.push(ch);
                                        } else {
                                            is_valid_low_hex = false;
                                            break;
                                        }
                                    }
                                    if is_valid_low_hex {
                                        if let Ok(low_code) = u32::from_str_radix(&low_hex, 16) {
                                            if (0xDC00..=0xDFFF).contains(&low_code) {
                                                self.pos += 6; // consume '\', 'u', and 4 hex digits
                                                let code_point = 0x10000 + (((code - 0xD800) << 10) | (low_code - 0xDC00));
                                                if let Some(unicode_char) = char::from_u32(code_point) {
                                                    s.push(unicode_char);
                                                }
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(unicode_char) = char::from_u32(code) {
                                s.push(unicode_char);
                            }
                        }
                    }
                    Some(escaped) => s.push(escaped),
                    None => return Err("Unexpected EOF in escape sequence".to_string()),
                }
            } else {
                s.push(c);
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<Object, String> {
        self.skip_whitespace();
        let start = self.pos;
        let mut is_float = false;

        if let Some('-') = self.peek() {
            self.next_char();
        }

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.next_char();
            } else if c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                if c == '.' || c == 'e' || c == 'E' {
                    is_float = true;
                }
                self.next_char();
            } else {
                break;
            }
        }

        let num_str: String = self.chars[start..self.pos].iter().collect();
        if is_float {
            match num_str.parse::<f64>() {
                Ok(f) => Ok(Object::Float(f)),
                Err(_) => Err(format!("Invalid float: {}", num_str)),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(i) => Ok(Object::Integer(i)),
                Err(_) => match num_str.parse::<f64>() {
                    Ok(f) => Ok(Object::Float(f)),
                    Err(_) => Err(format!("Invalid integer: {}", num_str)),
                },
            }
        }
    }

    fn parse_bool(&mut self) -> Result<Object, String> {
        self.skip_whitespace();
        if self.chars[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
            self.pos += 4;
            Ok(Object::Boolean(true))
        } else if self.chars[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
            self.pos += 5;
            Ok(Object::Boolean(false))
        } else {
            Err("Expected boolean".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<Object, String> {
        self.skip_whitespace();
        if self.chars[self.pos..].starts_with(&['n', 'u', 'l', 'l']) {
            self.pos += 4;
            Ok(Object::Null)
        } else {
            Err("Expected null".to_string())
        }
    }
}

pub fn parse_json(input: &str) -> Result<Object, String> {
    let mut parser = JsonParser::new(input);
    let val = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.pos != parser.chars.len() {
        return Err(format!("Trailing characters after JSON value at {}", parser.pos));
    }
    Ok(val)
}
