use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::path::{Component, Path, PathBuf};
use crate::object::{HashKey, Object};

fn make_ok(val: Object) -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("ok".to_string()), Object::Boolean(true));
    map.insert(HashKey::String("val".to_string()), val);
    map.insert(HashKey::String("err".to_string()), Object::Null);
    Object::Hash(Rc::new(RefCell::new(map)))
}

fn make_err(msg: &str) -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("ok".to_string()), Object::Boolean(false));
    map.insert(HashKey::String("val".to_string()), Object::Null);
    map.insert(HashKey::String("err".to_string()), Object::String(msg.to_string()));
    Object::Hash(Rc::new(RefCell::new(map)))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else if components.is_empty() || !matches!(components.last(), Some(Component::RootDir | Component::Prefix(_))) {
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let config = crate::stdlib::get_config();
    if !config.allow_fs {
        return Err("permission denied: filesystem access is disabled".to_string());
    }
    let p = Path::new(path);
    if let Some(ref root) = config.fs_root {
        let abs_path = if p.is_absolute() {
            p.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(p)
        };
        let normalized = normalize_path(&abs_path);
        let canon_root = root.canonicalize().unwrap_or_else(|_| normalize_path(root));

        let mut canon_base = PathBuf::new();
        let mut unexisting = PathBuf::new();
        let mut matched = false;

        for ancestor in normalized.ancestors() {
            if let Ok(canon_ancestor) = ancestor.canonicalize() {
                if !canon_ancestor.starts_with(&canon_root) {
                    return Err(format!("permission denied: path {:?} is outside sandboxed root {:?}", path, root));
                }
                canon_base = canon_ancestor;
                unexisting = normalized.strip_prefix(ancestor).unwrap().to_path_buf();
                matched = true;
                break;
            }
        }

        if !matched {
            return Err(format!("permission denied: path {:?} is outside sandboxed root {:?}", path, root));
        }

        // Ensure no components in the unexisting suffix are dangling symlinks.
        let mut current_check = canon_base.clone();
        for comp in unexisting.components() {
            current_check.push(comp);
            if current_check.is_symlink() {
                return Err(format!("permission denied: dangling symlink in path {:?}", path));
            }
        }

        return Ok(canon_base.join(unexisting));
    }
    Ok(p.to_path_buf())
}

pub fn make_module() -> Object {
    let mut map = HashMap::new();
    map.insert(HashKey::String("read_file".to_string()), Object::Builtin("std:fs:read_file".to_string()));
    map.insert(HashKey::String("read_file_or_throw".to_string()), Object::Builtin("std:fs:read_file_or_throw".to_string()));
    map.insert(HashKey::String("write_file".to_string()), Object::Builtin("std:fs:write_file".to_string()));
    map.insert(HashKey::String("write_file_or_throw".to_string()), Object::Builtin("std:fs:write_file_or_throw".to_string()));
    map.insert(HashKey::String("append_file".to_string()), Object::Builtin("std:fs:append_file".to_string()));
    map.insert(HashKey::String("append_file_or_throw".to_string()), Object::Builtin("std:fs:append_file_or_throw".to_string()));
    map.insert(HashKey::String("exists".to_string()), Object::Builtin("std:fs:exists".to_string()));
    map.insert(HashKey::String("remove_file".to_string()), Object::Builtin("std:fs:remove_file".to_string()));
    map.insert(HashKey::String("remove_file_or_throw".to_string()), Object::Builtin("std:fs:remove_file_or_throw".to_string()));
    map.insert(HashKey::String("create_dir".to_string()), Object::Builtin("std:fs:create_dir".to_string()));
    map.insert(HashKey::String("create_dir_or_throw".to_string()), Object::Builtin("std:fs:create_dir_or_throw".to_string()));
    Object::Hash(Rc::new(RefCell::new(map)))
}

pub fn apply(name: &str, args: Vec<Object>) -> Object {
    let config = crate::stdlib::get_config();
    match name {
        "read_file" => {
            if args.len() != 1 {
                return make_err(&format!("read_file expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(path) => {
                    if let Err(e) = validate_path(path) {
                        return make_err(&e);
                    }
                    if let Ok(meta) = fs::metadata(path) {
                        if meta.len() as usize > config.max_file_size {
                            return make_err("file size exceeds maximum allowed limit");
                        }
                    }
                    match fs::read_to_string(path) {
                        Ok(content) => make_ok(Object::String(content)),
                        Err(e) => make_err(&e.to_string()),
                    }
                }
                _ => make_err("read_file expects string path"),
            }
        }
        "read_file_or_throw" => {
            if args.len() != 1 {
                return Object::Error(format!("read_file_or_throw expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(path) => {
                    if let Err(e) = validate_path(path) {
                        return Object::Error(e);
                    }
                    if let Ok(meta) = fs::metadata(path) {
                        if meta.len() as usize > config.max_file_size {
                            return Object::Error("file size exceeds maximum allowed limit".to_string());
                        }
                    }
                    match fs::read_to_string(path) {
                        Ok(content) => Object::String(content),
                        Err(e) => Object::Error(e.to_string()),
                    }
                }
                _ => Object::Error("read_file_or_throw expects string path".to_string()),
            }
        }
        "write_file" => {
            if args.len() != 2 {
                return make_err(&format!("write_file expects 2 arguments, got {}", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(path), Object::String(content)) => {
                    if let Err(e) = validate_path(path) {
                        return make_err(&e);
                    }
                    if content.len() > config.max_file_size {
                        return make_err("content size exceeds maximum allowed limit");
                    }
                    match fs::write(path, content) {
                        Ok(_) => make_ok(Object::Boolean(true)),
                        Err(e) => make_err(&e.to_string()),
                    }
                }
                _ => make_err("write_file expects (string, string)"),
            }
        }
        "write_file_or_throw" => {
            if args.len() != 2 {
                return Object::Error(format!("write_file_or_throw expects 2 arguments, got {}", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(path), Object::String(content)) => {
                    if let Err(e) = validate_path(path) {
                        return Object::Error(e);
                    }
                    if content.len() > config.max_file_size {
                        return Object::Error("content size exceeds maximum allowed limit".to_string());
                    }
                    match fs::write(path, content) {
                        Ok(_) => Object::Boolean(true),
                        Err(e) => Object::Error(e.to_string()),
                    }
                }
                _ => Object::Error("write_file_or_throw expects (string, string)".to_string()),
            }
        }
        "append_file" => {
            if args.len() != 2 {
                return make_err(&format!("append_file expects 2 arguments, got {}", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(path), Object::String(content)) => {
                    if let Err(e) = validate_path(path) {
                        return make_err(&e);
                    }
                    if content.len() > config.max_file_size {
                        return make_err("content size exceeds maximum allowed limit");
                    }
                    use std::io::Write;
                    match fs::OpenOptions::new().create(true).append(true).open(path) {
                        Ok(mut file) => match file.write_all(content.as_bytes()) {
                            Ok(_) => make_ok(Object::Boolean(true)),
                            Err(e) => make_err(&e.to_string()),
                        },
                        Err(e) => make_err(&e.to_string()),
                    }
                }
                _ => make_err("append_file expects (string, string)"),
            }
        }
        "append_file_or_throw" => {
            if args.len() != 2 {
                return Object::Error(format!("append_file_or_throw expects 2 arguments, got {}", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(path), Object::String(content)) => {
                    if let Err(e) = validate_path(path) {
                        return Object::Error(e);
                    }
                    if content.len() > config.max_file_size {
                        return Object::Error("content size exceeds maximum allowed limit".to_string());
                    }
                    use std::io::Write;
                    match fs::OpenOptions::new().create(true).append(true).open(path) {
                        Ok(mut file) => match file.write_all(content.as_bytes()) {
                            Ok(_) => Object::Boolean(true),
                            Err(e) => Object::Error(e.to_string()),
                        },
                        Err(e) => Object::Error(e.to_string()),
                    }
                }
                _ => Object::Error("append_file_or_throw expects (string, string)".to_string()),
            }
        }
        "exists" => {
            if args.len() != 1 {
                return make_err(&format!("exists expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(path) => {
                    if let Err(e) = validate_path(path) {
                        return make_err(&e);
                    }
                    let exists = Path::new(path).exists();
                    make_ok(Object::Boolean(exists))
                }
                _ => make_err("exists expects string path"),
            }
        }
        "remove_file" => {
            if args.len() != 1 {
                return make_err(&format!("remove_file expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(path) => {
                    if let Err(e) = validate_path(path) {
                        return make_err(&e);
                    }
                    match fs::remove_file(path) {
                        Ok(_) => make_ok(Object::Boolean(true)),
                        Err(e) => make_err(&e.to_string()),
                    }
                }
                _ => make_err("remove_file expects string path"),
            }
        }
        "remove_file_or_throw" => {
            if args.len() != 1 {
                return Object::Error(format!("remove_file_or_throw expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(path) => {
                    if let Err(e) = validate_path(path) {
                        return Object::Error(e);
                    }
                    match fs::remove_file(path) {
                        Ok(_) => Object::Boolean(true),
                        Err(e) => Object::Error(e.to_string()),
                    }
                }
                _ => Object::Error("remove_file_or_throw expects string path".to_string()),
            }
        }
        "create_dir" => {
            if args.len() != 1 {
                return make_err(&format!("create_dir expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(path) => {
                    if let Err(e) = validate_path(path) {
                        return make_err(&e);
                    }
                    match fs::create_dir_all(path) {
                        Ok(_) => make_ok(Object::Boolean(true)),
                        Err(e) => make_err(&e.to_string()),
                    }
                }
                _ => make_err("create_dir expects string path"),
            }
        }
        "create_dir_or_throw" => {
            if args.len() != 1 {
                return Object::Error(format!("create_dir_or_throw expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Object::String(path) => {
                    if let Err(e) = validate_path(path) {
                        return Object::Error(e);
                    }
                    match fs::create_dir_all(path) {
                        Ok(_) => Object::Boolean(true),
                        Err(e) => Object::Error(e.to_string()),
                    }
                }
                _ => Object::Error("create_dir_or_throw expects string path".to_string()),
            }
        }
        _ => make_err(&format!("unknown std:fs function '{}'", name)),
    }
}
