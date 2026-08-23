use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::{Expression, Program, Statement};
use crate::object::{Environment, Object};

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
    }
}

fn eval_block_statement(statements: Vec<Statement>, env: Rc<RefCell<Environment>>) -> Object {
    let mut result = Object::Null;

    for statement in statements {
        result = eval_statement(statement, Rc::clone(&env));

        if let Object::ReturnValue(_) | Object::Error(_) = result {
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
                if let Object::ReturnValue(_) | Object::Error(_) = result {
                    return result; 
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
                        if let Object::ReturnValue(_) | Object::Error(_) = result {
                            return result; 
                        }
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

            Object::Null // if no match is found, return null
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

fn apply_builtin(name: &str, args: Vec<Object>) -> Object {
    match name {
        "len" => {
            if args.len() != 1 {
                return Object::Error(format!("wrong number of arguments. got={}, want=1", args.len()));
            }
            match &args[0] {
                Object::String(s) => Object::Integer(s.len() as i64),
                Object::Array(elements) => Object::Integer(elements.len() as i64),
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
        "<" => Object::Boolean(left < right),
        ">" => Object::Boolean(left > right),
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
        "<" => Object::Boolean(left < right),
        ">" => Object::Boolean(left > right),
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
