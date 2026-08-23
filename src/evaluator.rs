use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use crate::ast::{Expression, Program, Statement};
use crate::object::{Environment, HashKey, Object};
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::HashMap;

pub fn eval_program(program: Program, env: Rc<RefCell<Environment>>) -> Object {
    let mut result = Object::Null;

    for statement in program.statements {
        result = eval_statement(statement, Rc::clone(&env));

        match result {
            Object::ReturnValue(val) => return *val,
            Object::Error(_) => return result,
            _ => {}
        }
    }

    result
}

fn eval_statement(statement: Statement, env: Rc<RefCell<Environment>>) -> Object {
    match statement {
        Statement::Let { name, value, is_mutable } => {
            let val = eval_expression(value, Rc::clone(&env));
            if let Object::Error(_) = val {
                return val;
            }
            env.borrow_mut().set(name, val, is_mutable);
            Object::Null
        }
        Statement::Assign { name, value } => {
            let val = eval_expression(value, Rc::clone(&env));
            if let Object::Error(_) = val {
                return val;
            }
            match env.borrow_mut().assign(&name, val) {
                Ok(_) => Object::Null,
                Err(msg) => Object::Error(msg),
            }
        }
        Statement::IndexAssign { left, index, value } => {
            let index_val = eval_expression(index, Rc::clone(&env));
            if let Object::Error(_) = index_val {
                return index_val;
            }
            let val = eval_expression(value, Rc::clone(&env));
            if let Object::Error(_) = val {
                return val;
            }
            let target = eval_expression(left, env);
            if let Object::Error(_) = target {
                return target;
            }

            match target {
                Object::Array(rc) => {
                    let idx = match index_val {
                        Object::Integer(i) => i,
                        _ => return Object::Error(format!("array index must be integer, got {}", index_val.type_name())),
                    };
                    let mut vec = rc.borrow_mut();
                    if idx >= 0 && (idx as usize) < vec.len() {
                        vec[idx as usize] = val;
                    } else if idx >= 0 && (idx as usize) == vec.len() {
                        vec.push(val);
                    } else {
                        return Object::Error(format!("array index out of bounds: {}", idx));
                    }
                    Object::Null
                }
                Object::Hash(rc) => {
                    let hash_key = match index_val.get_hash_key() {
                        Ok(k) => k,
                        Err(msg) => return Object::Error(msg),
                    };
                    rc.borrow_mut().insert(hash_key, val);
                    Object::Null
                }
                Object::StructInstance { struct_name, fields } => {
                    let field_name = match index_val {
                        Object::String(s) => s,
                        _ => return Object::Error(format!("struct field index must be string, got {}", index_val.type_name())),
                    };
                    let mut map = fields.borrow_mut();
                    if !map.contains_key(&field_name) {
                        return Object::Error(format!("struct '{}' has no field '{}'", struct_name, field_name));
                    }
                    map.insert(field_name, val);
                    Object::Null
                }
                _ => Object::Error(format!("cannot index assign to type {}", target.type_name())),
            }
        }
        Statement::FieldAssign { object, field, value } => {
            let val = eval_expression(value, Rc::clone(&env));
            if let Object::Error(_) = val {
                return val;
            }
            let target = eval_expression(object, env);
            if let Object::Error(_) = target {
                return target;
            }

            match target {
                Object::StructInstance { struct_name, fields } => {
                    let mut map = fields.borrow_mut();
                    if !map.contains_key(&field) {
                        return Object::Error(format!("struct '{}' has no field '{}'", struct_name, field));
                    }
                    map.insert(field, val);
                    Object::Null
                }
                Object::Hash(rc) => {
                    rc.borrow_mut().insert(HashKey::String(field), val);
                    Object::Null
                }
                _ => Object::Error(format!("cannot assign field '{}' on type {}", field, target.type_name())),
            }
        }
        Statement::StructDef { name, fields } => {
            let struct_def = Object::StructDef {
                name: name.clone(),
                fields,
            };
            env.borrow_mut().set(name, struct_def, false);
            Object::Null
        }
        Statement::Return(expr) => {
            let val = eval_expression(expr, env);
            if let Object::Error(_) = val {
                return val;
            }
            Object::ReturnValue(Box::new(val))
        }
        Statement::Expression(expr) => eval_expression(expr, env),
        Statement::Block(statements) => eval_block_statement(statements, env),
        Statement::Break => Object::Break,
        Statement::Continue => Object::Continue,
    }
}

fn eval_block_statement(statements: Vec<Statement>, env: Rc<RefCell<Environment>>) -> Object {
    let mut result = Object::Null;

    for statement in statements {
        result = eval_statement(statement, Rc::clone(&env));

        if let Object::ReturnValue(_) | Object::Error(_) | Object::Break | Object::Continue = result {
            return result; 
        }
    }

    result
}

fn eval_expression(expression: Expression, env: Rc<RefCell<Environment>>) -> Object {
    match expression {
        Expression::IntegerLiteral(val) => Object::Integer(val),
        Expression::FloatLiteral(val) => Object::Float(val),
        Expression::Boolean(val) => Object::Boolean(val),
        Expression::StringLiteral(val) => Object::String(val),
        Expression::NullLiteral => Object::Null,
        Expression::Range { start, end, inclusive } => {
            let start_obj = eval_expression(*start, Rc::clone(&env));
            if let Object::Error(_) = start_obj {
                return start_obj;
            }
            let end_obj = eval_expression(*end, env);
            if let Object::Error(_) = end_obj {
                return end_obj;
            }
            match (start_obj, end_obj) {
                (Object::Integer(s), Object::Integer(e)) => Object::Range {
                    start: s,
                    end: e,
                    inclusive,
                },
                (s, e) => Object::Error(format!("range bounds must be integers, got {} and {}", s, e)),
            }
        }
        Expression::Array(elements) => {
            let mut eval_elements = Vec::new();
            for el in elements {
                let evaluated = eval_expression(el, Rc::clone(&env));
                if let Object::Error(_) = evaluated {
                    return evaluated;
                }
                eval_elements.push(evaluated);
            }
            Object::Array(Rc::new(RefCell::new(eval_elements)))
        }
        Expression::HashLiteral(pairs) => {
            let mut eval_pairs = HashMap::new();
            for (k, v) in pairs {
                let key_eval = eval_expression(k, Rc::clone(&env));
                if let Object::Error(_) = key_eval {
                    return key_eval;
                }
                
                let hash_key = match key_eval.get_hash_key() {
                    Ok(hk) => hk,
                    Err(msg) => return Object::Error(msg),
                };
                
                let val_eval = eval_expression(v, Rc::clone(&env));
                if let Object::Error(_) = val_eval {
                    return val_eval;
                }
                
                eval_pairs.insert(hash_key, val_eval);
            }
            Object::Hash(Rc::new(RefCell::new(eval_pairs)))
        }
        Expression::FieldAccess { object, field } => {
            let target = eval_expression(*object, env);
            if let Object::Error(_) = target {
                return target;
            }

            match target {
                Object::StructInstance { struct_name, fields } => {
                    let map = fields.borrow();
                    match map.get(&field) {
                        Some(val) => val.clone(),
                        None => Object::Error(format!("field '{}' not found on struct '{}'", field, struct_name)),
                    }
                }
                Object::Hash(rc) => {
                    let map = rc.borrow();
                    match map.get(&HashKey::String(field.clone())) {
                        Some(val) => val.clone(),
                        None => Object::Null,
                    }
                }
                _ => Object::Error(format!("cannot access field '{}' on type {}", field, target.type_name())),
            }
        }
        Expression::Index { left, index } => {
            let left_evaluated = eval_expression(*left, Rc::clone(&env));
            if let Object::Error(_) = left_evaluated {
                return left_evaluated;
            }
            let index_evaluated = eval_expression(*index, env);
            if let Object::Error(_) = index_evaluated {
                return index_evaluated;
            }
            eval_index_expression(left_evaluated, index_evaluated)
        }
        Expression::Identifier(name) => {
            if let Some(val) = env.borrow().get(&name) {
                val
            } else {
                Object::Error(format!("identifier not found: {}", name))
            }
        }
        Expression::Prefix { operator, right } => {
            let right_evaluated = eval_expression(*right, env);
            if let Object::Error(_) = right_evaluated {
                return right_evaluated;
            }
            eval_prefix_expression(&operator, right_evaluated)
        }
        Expression::Infix { left, operator, right } => {
            if operator == "&&" || operator == "||" {
                let left_evaluated = eval_expression(*left, Rc::clone(&env));
                if let Object::Error(_) = left_evaluated {
                    return left_evaluated;
                }
                let left_truthy = is_truthy(&left_evaluated);
                
                if operator == "&&" && !left_truthy {
                    return Object::Boolean(false);
                }
                if operator == "||" && left_truthy {
                    return Object::Boolean(true);
                }
                
                let right_evaluated = eval_expression(*right, env);
                if let Object::Error(_) = right_evaluated {
                    return right_evaluated;
                }
                return Object::Boolean(is_truthy(&right_evaluated));
            }

            let left_evaluated = eval_expression(*left, Rc::clone(&env));
            if let Object::Error(_) = left_evaluated {
                return left_evaluated;
            }
            
            let right_evaluated = eval_expression(*right, env);
            if let Object::Error(_) = right_evaluated {
                return right_evaluated;
            }
            
            eval_infix_expression(&operator, left_evaluated, right_evaluated)
        }
        Expression::If { condition, consequence, alternative } => {
            let cond_evaluated = eval_expression(*condition, Rc::clone(&env));
            if let Object::Error(_) = cond_evaluated {
                return cond_evaluated;
            }
            
            if is_truthy(&cond_evaluated) {
                eval_statement(*consequence, env)
            } else if let Some(alt) = alternative {
                eval_statement(*alt, env)
            } else {
                Object::Null
            }
        }
        Expression::While { condition, body } => {
            loop {
                let cond_evaluated = eval_expression(*condition.clone(), Rc::clone(&env));
                if let Object::Error(_) = cond_evaluated {
                    return cond_evaluated;
                }
                
                if !is_truthy(&cond_evaluated) {
                    break;
                }
                
                let result = eval_statement(*body.clone(), Rc::clone(&env));
                match result {
                    Object::ReturnValue(_) | Object::Error(_) => return result,
                    Object::Break => break,
                    Object::Continue => continue,
                    _ => {}
                }
            }
            Object::Null
        }
        Expression::For { variable, iterable, body } => {
            let iter_evaluated = eval_expression(*iterable, Rc::clone(&env));
            if let Object::Error(_) = iter_evaluated {
                return iter_evaluated;
            }
            
            match iter_evaluated {
                Object::Array(rc) => {
                    let elements = rc.borrow().clone();
                    for el in elements {
                        let loop_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));
                        loop_env.borrow_mut().set(variable.clone(), el, false); 
                        
                        let result = eval_statement(*body.clone(), loop_env);
                        match result {
                            Object::ReturnValue(_) | Object::Error(_) => return result,
                            Object::Break => break,
                            Object::Continue => continue,
                            _ => {}
                        }
                    }
                    Object::Null
                }
                Object::Range { start, end, inclusive } => {
                    let mut curr = start;
                    while if inclusive { curr <= end } else { curr < end } {
                        let loop_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));
                        loop_env.borrow_mut().set(variable.clone(), Object::Integer(curr), false); 
                        
                        let result = eval_statement(*body.clone(), loop_env);
                        match result {
                            Object::ReturnValue(_) | Object::Error(_) => return result,
                            Object::Break => break,
                            Object::Continue => {
                                curr = match curr.checked_add(1) {
                                    Some(v) => v,
                                    None => break,
                                };
                                continue;
                            }
                            _ => {}
                        }
                        curr = match curr.checked_add(1) {
                            Some(v) => v,
                            None => break,
                        };
                    }
                    Object::Null
                }
                _ => Object::Error(format!("cannot iterate over: {}", iter_evaluated)),
            }
        }
        Expression::Match { value, cases } => {
            let val_evaluated = eval_expression(*value, Rc::clone(&env));
            if let Object::Error(_) = val_evaluated {
                return val_evaluated;
            }

            for (pattern, body) in cases {
                let is_catch_all = if let Expression::Identifier(name) = &pattern {
                    name == "_"
                } else {
                    false
                };

                if is_catch_all {
                    return eval_statement(*body, env);
                }

                let pattern_evaluated = eval_expression(pattern, Rc::clone(&env));
                if let Object::Error(_) = pattern_evaluated {
                    return pattern_evaluated;
                }

                if val_evaluated == pattern_evaluated {
                    return eval_statement(*body, env);
                }
            }
            Object::Null
        }
        Expression::Throw(exp) => {
            let result = eval_expression(*exp, env);
            if let Object::Error(_) = result {
                return result;
            }
            Object::Error(format!("{}", result))
        }
        Expression::TryCatch { try_body, catch_param, catch_body } => {
            let result = eval_statement(*try_body, Rc::clone(&env));
            if let Object::Error(msg) = result {
                let catch_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
                catch_env.borrow_mut().set(catch_param, Object::String(msg), false);
                eval_statement(*catch_body, catch_env)
            } else {
                result
            }
        }
        Expression::FunctionLiteral { parameters, return_type, body, .. } => {
            Object::Function {
                parameters,
                return_type,
                body: *body,
                env,
            }
        }
        Expression::Call { function, arguments } => {
            let func = eval_expression(*function, Rc::clone(&env));
            if let Object::Error(_) = func {
                return func;
            }

            let mut args = Vec::new();
            for arg in arguments {
                let evaluated = eval_expression(arg, Rc::clone(&env));
                if let Object::Error(_) = evaluated {
                    return evaluated;
                }
                args.push(evaluated);
            }

            apply_function(func, args)
        }
    }
}

fn eval_index_expression(left: Object, index: Object) -> Object {
    match (left, index) {
        (Object::Array(rc), Object::Integer(idx)) => {
            let elements = rc.borrow();
            if idx < 0 || idx as usize >= elements.len() {
                Object::Null
            } else {
                elements[idx as usize].clone()
            }
        }
        (Object::Hash(rc), index_val) => {
            let hash_key = match index_val.get_hash_key() {
                Ok(hk) => hk,
                Err(msg) => return Object::Error(msg),
            };
            
            let pairs = rc.borrow();
            match pairs.get(&hash_key) {
                Some(val) => val.clone(),
                None => Object::Null,
            }
        }
        (Object::StructInstance { struct_name, fields }, Object::String(field_name)) => {
            let map = fields.borrow();
            match map.get(&field_name) {
                Some(val) => val.clone(),
                None => Object::Error(format!("field '{}' not found on struct '{}'", field_name, struct_name)),
            }
        }
        (l, _) => Object::Error(format!("index operator not supported for: {}", l)),
    }
}

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(val) => *val,
        _ => true,
    }
}

fn apply_function(func: Object, args: Vec<Object>) -> Object {
    match func {
        Object::Function { parameters, return_type, body, env } => {
            let extended_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
            for (i, (param_name, param_type)) in parameters.iter().enumerate() {
                if i < args.len() {
                    let arg = &args[i];
                    if let Some(expected_type) = param_type {
                        let actual_type = arg.type_name();
                        if actual_type != *expected_type && expected_type != "Any" {
                            return Object::Error(format!("type mismatch for parameter '{}': expected {}, got {}", param_name, expected_type, actual_type));
                        }
                    }
                    extended_env.borrow_mut().set(param_name.clone(), arg.clone(), true);
                }
            }
            
            let evaluated = eval_statement(body, extended_env);
            let final_val = if let Object::ReturnValue(val) = evaluated {
                *val
            } else {
                evaluated
            };

            if let Some(expected_ret) = return_type {
                let actual_ret = final_val.type_name();
                if actual_ret != expected_ret && expected_ret != "Any" {
                    return Object::Error(format!("return type mismatch: expected {}, got {}", expected_ret, actual_ret));
                }
            }

            final_val
        }
        Object::StructDef { name, fields } => {
            if args.len() != fields.len() {
                return Object::Error(format!("struct '{}' expects {} arguments, got {}", name, fields.len(), args.len()));
            }
            let mut instance_fields = HashMap::new();
            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let arg = &args[i];
                if let Some(expected_type) = field_type {
                    let actual_type = arg.type_name();
                    if actual_type != *expected_type && expected_type != "Any" {
                        return Object::Error(format!("type mismatch for struct field '{}': expected {}, got {}", field_name, expected_type, actual_type));
                    }
                }
                instance_fields.insert(field_name.clone(), arg.clone());
            }
            Object::StructInstance {
                struct_name: name,
                fields: Rc::new(RefCell::new(instance_fields)),
            }
        }
        Object::Builtin(name) => apply_builtin(&name, args),
        _ => Object::Error(format!("not a function: {}", func)),
    }
}

pub fn apply_builtin(name: &str, args: Vec<Object>) -> Object {
    if name.starts_with("std:") {
        return match crate::stdlib::apply_std_builtin(name, args) {
            Some(res) => res,
            None => Object::Error(format!("unknown standard library function: {}", name)),
        };
    }

    match name {
        "len" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::String(s) => Object::Integer(s.chars().count() as i64),
                Object::Array(rc) => Object::Integer(rc.borrow().len() as i64),
                Object::Hash(rc) => Object::Integer(rc.borrow().len() as i64),
                Object::StructInstance { fields, .. } => Object::Integer(fields.borrow().len() as i64),
                Object::Range { start, end, inclusive } => {
                    if *end >= *start {
                        let count = if *inclusive { (*end - *start).saturating_add(1) } else { *end - *start };
                        Object::Integer(count)
                    } else {
                        Object::Integer(0)
                    }
                }
                _ => Object::Error(format!("argument to `len` not supported, got {}", args[0])),
            }
        }
        "push" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            match &args[0] {
                Object::Array(rc) => {
                    rc.borrow_mut().push(args[1].clone());
                    Object::Array(Rc::clone(rc))
                }
                _ => Object::Error(format!("argument to `push` must be ARRAY, got {}", args[0])),
            }
        }
        "pop" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::Array(rc) => {
                    rc.borrow_mut().pop().unwrap_or(Object::Null)
                }
                _ => Object::Error(format!("argument to `pop` must be ARRAY, got {}", args[0])),
            }
        }
        "print" => {
            for arg in args {
                println!("{}", arg);
            }
            Object::Null
        }
        "map" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            if let Object::Array(rc) = &args[0] {
                let elements = rc.borrow().clone();
                let func = &args[1];
                let mut mapped = Vec::new();
                for el in elements {
                    let result = apply_function(func.clone(), vec![el]);
                    if let Object::Error(_) = result {
                        return result;
                    }
                    mapped.push(result);
                }
                Object::Array(Rc::new(RefCell::new(mapped)))
            } else {
                Object::Error(format!("first argument to `map` must be ARRAY, got {}", args[0]))
            }
        }
        "filter" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            if let Object::Array(rc) = &args[0] {
                let elements = rc.borrow().clone();
                let func = &args[1];
                let mut filtered = Vec::new();
                for el in elements {
                    let result = apply_function(func.clone(), vec![el.clone()]);
                    if let Object::Error(_) = result {
                        return result;
                    }
                    if is_truthy(&result) {
                        filtered.push(el);
                    }
                }
                Object::Array(Rc::new(RefCell::new(filtered)))
            } else {
                Object::Error(format!("first argument to `filter` must be ARRAY, got {}", args[0]))
            }
        }
        "reduce" => {
            if args.len() != 3 {
                return Object::Error(format!("wrong number of arguments. got={}, want=3", args.len()));
            }
            if let Object::Array(rc) = &args[0] {
                let elements = rc.borrow().clone();
                let mut accumulator = args[1].clone();
                let func = &args[2];
                for el in elements {
                    accumulator = apply_function(func.clone(), vec![accumulator, el]);
                    if let Object::Error(_) = accumulator {
                        return accumulator;
                    }
                }
                accumulator
            } else {
                Object::Error(format!("first argument to `reduce` must be ARRAY, got {}", args[0]))
            }
        }
        "import" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments for import. got={}, want=1", args.len()));
            }
            if let Object::String(filename) = &args[0] {
                if filename.starts_with("std:") || filename.starts_with("std/") {
                    if let Some(module) = crate::stdlib::load_std_module(filename) {
                        return module;
                    } else {
                        return Object::Error(format!("unknown standard library module: {}", filename));
                    }
                }

                match fs::read_to_string(filename) {
                    Ok(contents) => {
                        let lexer = Lexer::new(&contents);
                        let mut parser = Parser::new(lexer);
                        let program = parser.parse_program();
                        
                        if !parser.errors.is_empty() {
                            return Object::Error(format!("parse errors in {}:\n{}", filename, parser.errors.join("\n")));
                        }
                        
                        let module_env = Rc::new(RefCell::new(Environment::new()));
                        let eval_result = eval_program(program, Rc::clone(&module_env));
                        if let Object::Error(_) = eval_result {
                            return eval_result;
                        }
                        
                        let mut exports = HashMap::new();
                        for (k, (v, _)) in module_env.borrow().get_all().iter() {
                            exports.insert(HashKey::String(k.clone()), v.clone());
                        }
                        
                        Object::Hash(Rc::new(RefCell::new(exports)))
                    }
                    Err(e) => Object::Error(format!("could not read file {}: {}", filename, e)),
                }
            } else {
                Object::Error(format!("argument to `import` must be STRING, got {}", args[0]))
            }
        }
        "split" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(s), Object::String(delim)) => {
                    let parts: Vec<Object> = s.split(delim.as_str()).map(|p| Object::String(p.to_string())).collect();
                    Object::Array(Rc::new(RefCell::new(parts)))
                }
                _ => Object::Error("arguments to `split` must be (STRING, STRING)".to_string()),
            }
        }
        "trim" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::String(s) => Object::String(s.trim().to_string()),
                _ => Object::Error(format!("argument to `trim` must be STRING, got {}", args[0])),
            }
        }
        "replace" => {
            if args.len() != 3 {
                return Object::Error(format!("wrong number of arguments. got={}, want=3", args.len()));
            }
            match (&args[0], &args[1], &args[2]) {
                (Object::String(s), Object::String(from), Object::String(to)) => {
                    Object::String(s.replace(from.as_str(), to.as_str()))
                }
                _ => Object::Error("arguments to `replace` must be (STRING, STRING, STRING)".to_string()),
            }
        }
        "join" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::Array(rc), Object::String(sep)) => {
                    let vec = rc.borrow();
                    let str_elements: Vec<String> = vec.iter().map(|e| e.to_string()).collect();
                    Object::String(str_elements.join(sep))
                }
                _ => Object::Error("arguments to `join` must be (ARRAY, STRING)".to_string()),
            }
        }
        "contains" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(s), Object::String(sub)) => {
                    Object::Boolean(s.contains(sub.as_str()))
                }
                _ => Object::Error("arguments to `contains` must be (STRING, STRING)".to_string()),
            }
        }
        "starts_with" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(s), Object::String(prefix)) => {
                    Object::Boolean(s.starts_with(prefix.as_str()))
                }
                _ => Object::Error("arguments to `starts_with` must be (STRING, STRING)".to_string()),
            }
        }
        "ends_with" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            match (&args[0], &args[1]) {
                (Object::String(s), Object::String(suffix)) => {
                    Object::Boolean(s.ends_with(suffix.as_str()))
                }
                _ => Object::Error("arguments to `ends_with` must be (STRING, STRING)".to_string()),
            }
        }
        "to_upper" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::String(s) => Object::String(s.to_uppercase()),
                _ => Object::Error(format!("argument to `to_upper` must be STRING, got {}", args[0])),
            }
        }
        "to_lower" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::String(s) => Object::String(s.to_lowercase()),
                _ => Object::Error(format!("argument to `to_lower` must be STRING, got {}", args[0])),
            }
        }
        "substring" => {
            if args.len() != 3 {
                return Object::Error(format!("wrong number of arguments. got={}, want=3", args.len()));
            }
            match (&args[0], &args[1], &args[2]) {
                (Object::String(s), Object::Integer(start), Object::Integer(end)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let char_len = chars.len() as i64;
                    let start_idx = (*start).max(0).min(char_len) as usize;
                    let end_idx = (*end).max(start_idx as i64).min(char_len) as usize;
                    let sub: String = chars[start_idx..end_idx].iter().collect();
                    Object::String(sub)
                }
                _ => Object::Error("arguments to `substring` must be (STRING, INT, INT)".to_string()),
            }
        }
        _ => Object::Error(format!("builtin function not found: {}", name)),
    }
}

fn eval_prefix_expression(operator: &str, right: Object) -> Object {
    match operator {
        "-" => eval_minus_prefix_operator_expression(right),
        "!" => eval_bang_prefix_operator_expression(right),
        _ => Object::Error(format!("unknown operator: {}{}", operator, right)),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Object {
    match right {
        Object::Integer(val) => Object::Integer(-val),
        Object::Float(val) => Object::Float(-val),
        _ => Object::Error(format!("unknown operator: -{}", right)),
    }
}

fn eval_bang_prefix_operator_expression(right: Object) -> Object {
    Object::Boolean(!is_truthy(&right))
}

fn eval_infix_expression(operator: &str, left: Object, right: Object) -> Object {
    if operator == "==" {
        return Object::Boolean(left == right);
    }
    if operator == "!=" {
        return Object::Boolean(left != right);
    }

    if operator == "+" {
        if let (Object::String(l), Object::String(r)) = (&left, &right) {
            return eval_string_infix_expression(operator, l.clone(), r.clone());
        }
        if matches!(left, Object::String(_)) || matches!(right, Object::String(_)) {
            return Object::String(format!("{}{}", left, right));
        }
    }

    match (left, right) {
        (Object::Integer(left_val), Object::Integer(right_val)) => {
            eval_integer_infix_expression(operator, left_val, right_val)
        }
        (Object::Float(left_val), Object::Float(right_val)) => {
            eval_float_infix_expression(operator, left_val, right_val)
        }
        (Object::Float(left_val), Object::Integer(right_val)) => {
            eval_float_infix_expression(operator, left_val, right_val as f64)
        }
        (Object::Integer(left_val), Object::Float(right_val)) => {
            eval_float_infix_expression(operator, left_val as f64, right_val)
        }
        (Object::Boolean(left_val), Object::Boolean(right_val)) => {
            eval_boolean_infix_expression(operator, left_val, right_val)
        }
        (Object::String(left_val), Object::String(right_val)) => {
            eval_string_infix_expression(operator, left_val, right_val)
        }
        (left, right) => Object::Error(format!("type mismatch: {} {} {}", left, operator, right)),
    }
}

fn eval_string_infix_expression(operator: &str, left: String, right: String) -> Object {
    match operator {
        "+" => Object::String(format!("{}{}", left, right)),
        "==" => Object::Boolean(left == right),
        "!=" => Object::Boolean(left != right),
        _ => Object::Error(format!("unknown operator: {} {} {}", left, operator, right)),
    }
}

fn eval_float_infix_expression(operator: &str, left: f64, right: f64) -> Object {
    match operator {
        "+" => Object::Float(left + right),
        "-" => Object::Float(left - right),
        "*" => Object::Float(left * right),
        "/" => {
            if right == 0.0 {
                Object::Error("division by zero".to_string())
            } else {
                Object::Float(left / right)
            }
        }
        "%" => {
            if right == 0.0 {
                Object::Error("division by zero".to_string())
            } else {
                Object::Float(left % right)
            }
        }
        "<" => Object::Boolean(left < right),
        ">" => Object::Boolean(left > right),
        "<=" => Object::Boolean(left <= right),
        ">=" => Object::Boolean(left >= right),
        "==" => Object::Boolean(left == right),
        "!=" => Object::Boolean(left != right),
        _ => Object::Error(format!("unknown operator: {} {} {}", left, operator, right)),
    }
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Object {
    match operator {
        "+" => Object::Integer(left + right),
        "-" => Object::Integer(left - right),
        "*" => Object::Integer(left * right),
        "/" => {
            if right == 0 {
                Object::Error("division by zero".to_string())
            } else {
                Object::Integer(left / right)
            }
        }
        "%" => {
            if right == 0 {
                Object::Error("division by zero".to_string())
            } else {
                Object::Integer(left % right)
            }
        }
        "<" => Object::Boolean(left < right),
        ">" => Object::Boolean(left > right),
        "<=" => Object::Boolean(left <= right),
        ">=" => Object::Boolean(left >= right),
        "==" => Object::Boolean(left == right),
        "!=" => Object::Boolean(left != right),
        _ => Object::Error(format!("unknown operator: {} {} {}", left, operator, right)),
    }
}

fn eval_boolean_infix_expression(operator: &str, left: bool, right: bool) -> Object {
    match operator {
        "==" => Object::Boolean(left == right),
        "!=" => Object::Boolean(left != right),
        _ => Object::Error(format!("unknown operator: {} {} {}", left, operator, right)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        let env = Rc::new(RefCell::new(Environment::new()));
        eval_program(program, env)
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            if let Object::Integer(val) = evaluated {
                assert_eq!(val, expected);
            } else {
                panic!("object is not Integer. got={:?}", evaluated);
            }
        }
    }

    #[test]
    fn test_eval_compound_assignments() {
        let input = "
            var a = 10
            a += 5
            a -= 3
            a *= 2
            a /= 4
            a %= 5
            a
        ";
        let evaluated = test_eval(input);
        assert_eq!(evaluated, Object::Integer(1));
    }

    #[test]
    fn test_eval_container_mutation() {
        let input = "
            var arr = [1, 2, 3]
            arr[1] = 99
            arr[1]
        ";
        assert_eq!(test_eval(input), Object::Integer(99));

        let input_matrix = "
            var matrix = [[1, 2], [3, 4]]
            matrix[1][0] = 42
            matrix[1][0]
        ";
        assert_eq!(test_eval(input_matrix), Object::Integer(42));

        let input_swap = "
            func swap(arr: Array, i: Int, j: Int) {
                let temp = arr[i]
                arr[i] = arr[j]
                arr[j] = temp
            }
            var numbers = [10, 20, 30]
            swap(numbers, 0, 2)
            numbers[0]
        ";
        assert_eq!(test_eval(input_swap), Object::Integer(30));
    }

    #[test]
    fn test_eval_structs_and_dot_notation() {
        let input = r#"
            struct Point {
                x: Int,
                y: Int
            }
            var p = Point(10, 20)
            p.x = 99
            p.x + p.y
        "#;
        assert_eq!(test_eval(input), Object::Integer(119));

        let input_dict = r#"
            var settings = {"theme": "dark", "zoom": 100}
            settings.theme = "light"
            settings.theme
        "#;
        assert_eq!(test_eval(input_dict), Object::String("light".to_string()));
    }

    #[test]
    fn test_eval_stdlib_math_json() {
        let input_math = r#"
            let math = import("std:math")
            math.sqrt(16.0)
        "#;
        assert_eq!(test_eval(input_math), Object::Float(4.0));

        let input_json = r#"
            let json = import("std:json")
            let parsed = json.parse("\{ \"a\": 10, \"b\": true \}")
            parsed.a
        "#;
        assert_eq!(test_eval(input_json), Object::Integer(10));
    }
}
