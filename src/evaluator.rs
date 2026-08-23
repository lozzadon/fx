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
            Object::Array(eval_elements)
        }
        Expression::HashLiteral(pairs) => {
            let mut eval_pairs = std::collections::HashMap::new();
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
            Object::Hash(eval_pairs)
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
                Object::Array(elements) => {
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
                // Check if it's the `_` catch-all pattern
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

                // If pattern matches, evaluate body
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
                result // Return the result of try_body if no error
            }
        }
        Expression::FunctionLiteral { parameters, return_type, body, .. } => {
            Object::Function {
                parameters,
                return_type,
                body: *body,
                env, // Takes ownership of the clone
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
        (Object::Array(elements), Object::Integer(idx)) => {
            if idx < 0 || idx as usize >= elements.len() {
                Object::Null
            } else {
                elements[idx as usize].clone()
            }
        }
        (Object::Hash(pairs), index_val) => {
            let hash_key = match index_val.get_hash_key() {
                Ok(hk) => hk,
                Err(msg) => return Object::Error(msg),
            };
            
            match pairs.get(&hash_key) {
                Some(val) => val.clone(),
                None => Object::Null,
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

            // Check return type
            if let Some(expected_ret) = return_type {
                let actual_ret = final_val.type_name();
                if actual_ret != expected_ret && expected_ret != "Any" {
                    return Object::Error(format!("return type mismatch: expected {}, got {}", expected_ret, actual_ret));
                }
            }

            final_val
        }
        Object::Builtin(name) => apply_builtin(&name, args),
        _ => Object::Error(format!("not a function: {}", func)),
    }
}

pub fn apply_builtin(name: &str, args: Vec<Object>) -> Object {
    match name {
        "len" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::String(s) => Object::Integer(s.chars().count() as i64),
                Object::Array(elements) => Object::Integer(elements.len() as i64),
                Object::Hash(pairs) => Object::Integer(pairs.len() as i64),
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
                Object::Array(elements) => {
                    let mut new_elements = elements.clone();
                    new_elements.push(args[1].clone());
                    Object::Array(new_elements)
                }
                _ => Object::Error(format!("argument to `push` must be ARRAY, got {}", args[0])),
            }
        }
        "pop" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::Array(elements) => {
                    let mut new_elements = elements.clone();
                    new_elements.pop();
                    Object::Array(new_elements)
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
            if let Object::Array(elements) = &args[0] {
                let func = &args[1];
                let mut mapped = Vec::new();
                for el in elements {
                    let result = apply_function(func.clone(), vec![el.clone()]);
                    if let Object::Error(_) = result {
                        return result;
                    }
                    mapped.push(result);
                }
                Object::Array(mapped)
            } else {
                Object::Error(format!("first argument to `map` must be ARRAY, got {}", args[0]))
            }
        }
        "filter" => {
            if args.len() != 2 {
                return Object::Error(format!("wrong number of arguments. got={}, want=2", args.len()));
            }
            if let Object::Array(elements) = &args[0] {
                let func = &args[1];
                let mut filtered = Vec::new();
                for el in elements {
                    let result = apply_function(func.clone(), vec![el.clone()]);
                    if let Object::Error(_) = result {
                        return result;
                    }
                    if is_truthy(&result) {
                        filtered.push(el.clone());
                    }
                }
                Object::Array(filtered)
            } else {
                Object::Error(format!("first argument to `filter` must be ARRAY, got {}", args[0]))
            }
        }
        "reduce" => {
            if args.len() != 3 {
                return Object::Error(format!("wrong number of arguments. got={}, want=3", args.len()));
            }
            if let Object::Array(elements) = &args[0] {
                let mut accumulator = args[1].clone();
                let func = &args[2];
                for el in elements {
                    accumulator = apply_function(func.clone(), vec![accumulator, el.clone()]);
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
                        
                        Object::Hash(exports)
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
                    Object::Array(parts)
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
                (Object::Array(elements), Object::String(sep)) => {
                    let str_elements: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
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
        assert_eq!(evaluated, Object::Integer(1)); // (10 + 5 - 3) * 2 / 4 % 5 = 24 / 4 % 5 = 6 % 5 = 1
    }

    #[test]
    fn test_eval_relational_and_modulo() {
        let tests = vec![
            ("5 <= 10", Object::Boolean(true)),
            ("10 <= 10", Object::Boolean(true)),
            ("15 <= 10", Object::Boolean(false)),
            ("5 >= 10", Object::Boolean(false)),
            ("10 >= 10", Object::Boolean(true)),
            ("15 >= 10", Object::Boolean(true)),
            ("14 % 4", Object::Integer(2)),
            ("10.5 % 3.0", Object::Float(1.5)),
            ("10 <= 10.5", Object::Boolean(true)),
            ("11 >= 10.5", Object::Boolean(true)),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            assert_eq!(evaluated, expected, "Failed for input: {}", input);
        }

        let err_eval = test_eval("10 % 0");
        assert_eq!(err_eval, Object::Error("division by zero".to_string()));
    }

    #[test]
    fn test_eval_range_and_for_loops() {
        let input = "
            var sum = 0
            for i in 0..5 {
                sum += i
            }
            sum
        ";
        assert_eq!(test_eval(input), Object::Integer(10)); // 0+1+2+3+4 = 10

        let input_inclusive = "
            var fact = 1
            for i in 1..=5 {
                fact *= i
            }
            fact
        ";
        assert_eq!(test_eval(input_inclusive), Object::Integer(120)); // 1*2*3*4*5 = 120
    }

    #[test]
    fn test_eval_string_utilities() {
        let input_trim = r#"trim("  hello world  ")"#;
        assert_eq!(test_eval(input_trim), Object::String("hello world".to_string()));

        let input_split = r#"split("a,b,c", ",")"#;
        assert_eq!(
            test_eval(input_split),
            Object::Array(vec![
                Object::String("a".to_string()),
                Object::String("b".to_string()),
                Object::String("c".to_string()),
            ])
        );

        let input_replace = r#"replace("hello world", "world", "f(x)")"#;
        assert_eq!(test_eval(input_replace), Object::String("hello f(x)".to_string()));

        let input_join = r#"join(["apple", "banana", "cherry"], ", ")"#;
        assert_eq!(test_eval(input_join), Object::String("apple, banana, cherry".to_string()));

        let input_contains = r#"contains("rustacean", "ace")"#;
        assert_eq!(test_eval(input_contains), Object::Boolean(true));

        let input_starts = r#"starts_with("hello world", "hello")"#;
        assert_eq!(test_eval(input_starts), Object::Boolean(true));

        let input_ends = r#"ends_with("hello world", "world")"#;
        assert_eq!(test_eval(input_ends), Object::Boolean(true));

        let input_upper = r#"to_upper("hello")"#;
        assert_eq!(test_eval(input_upper), Object::String("HELLO".to_string()));

        let input_lower = r#"to_lower("HELLO")"#;
        assert_eq!(test_eval(input_lower), Object::String("hello".to_string()));

        let input_sub = r#"substring("hello world", 0, 5)"#;
        assert_eq!(test_eval(input_sub), Object::String("hello".to_string()));
    }
}

#[cfg(test)]
mod tests_control_flow {
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
    fn test_break_and_continue() {
        let input = "
            var results = []
            var i = 0
            while i < 10 {
                i = i + 1
                if i == 5 {
                    continue
                }
                if i == 8 {
                    break
                }
                results = push(results, i)
            }
            results
        ";

        let evaluated = test_eval(input);
        if let Object::Array(elements) = evaluated {
            assert_eq!(elements.len(), 6); // 1, 2, 3, 4, 6, 7
            assert_eq!(elements[0], Object::Integer(1));
            assert_eq!(elements[4], Object::Integer(6));
            assert_eq!(elements[5], Object::Integer(7));
        } else {
            panic!("Expected array, got {:?}", evaluated);
        }
    }
}
