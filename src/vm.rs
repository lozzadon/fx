use crate::compiler::Bytecode;
use crate::object::{Object, Environment};
use crate::code::{Opcode, Instructions};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

const STACK_SIZE: usize = 2048;

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(b) => *b,
        _ => true,
    }
}

pub struct VM {
    constants: Vec<Object>,
    instructions: Instructions,
    stack: Vec<Object>,
    sp: usize, // Stack pointer
    pub globals: Vec<Object>,
    pub symbol_names: Vec<String>,
    pub env: Rc<RefCell<Environment>>,
    pub last_popped: Option<Object>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> VM {
        VM {
            constants: bytecode.constants,
            instructions: bytecode.instructions,
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
            globals: vec![Object::Null; 65536],
            symbol_names: bytecode.symbol_names,
            env: bytecode.env,
            last_popped: None,
        }
    }

    pub fn last_popped_elem(&self) -> Option<&Object> {
        self.last_popped.as_ref()
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut ip = 0;
        
        while ip < self.instructions.len() {
            let op = Opcode::from(self.instructions[ip]);
            
            match op {
                Opcode::OpConstant => {
                    let const_index = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    ip += 2;
                    
                    let mut obj = self.constants[const_index].clone();
                    if let Object::Function { ref mut env, .. } = obj {
                        *env = Rc::clone(&self.env);
                    }
                    self.push(obj)?;
                }
                Opcode::OpAdd | Opcode::OpSub | Opcode::OpMul | Opcode::OpDiv | Opcode::OpModulo => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    
                    if let (Object::Integer(l), Object::Integer(r)) = (&left, &right) {
                        let result = match op {
                            Opcode::OpAdd => l + r,
                            Opcode::OpSub => l - r,
                            Opcode::OpMul => l * r,
                            Opcode::OpDiv => {
                                if *r == 0 { return Err("division by zero".to_string()); }
                                l / r
                            }
                            Opcode::OpModulo => {
                                if *r == 0 { return Err("division by zero".to_string()); }
                                l % r
                            }
                            _ => unreachable!(),
                        };
                        self.push(Object::Integer(result))?;
                    } else if let (Object::Float(l), Object::Float(r)) = (&left, &right) {
                        let result = match op {
                            Opcode::OpAdd => l + r,
                            Opcode::OpSub => l - r,
                            Opcode::OpMul => l * r,
                            Opcode::OpDiv => {
                                if *r == 0.0 { return Err("division by zero".to_string()); }
                                l / r
                            }
                            Opcode::OpModulo => {
                                if *r == 0.0 { return Err("division by zero".to_string()); }
                                l % r
                            }
                            _ => unreachable!(),
                        };
                        self.push(Object::Float(result))?;
                    } else if let (Object::Float(l), Object::Integer(r)) = (&left, &right) {
                        let r_f = *r as f64;
                        let result = match op {
                            Opcode::OpAdd => l + r_f,
                            Opcode::OpSub => l - r_f,
                            Opcode::OpMul => l * r_f,
                            Opcode::OpDiv => {
                                if r_f == 0.0 { return Err("division by zero".to_string()); }
                                l / r_f
                            }
                            Opcode::OpModulo => {
                                if r_f == 0.0 { return Err("division by zero".to_string()); }
                                l % r_f
                            }
                            _ => unreachable!(),
                        };
                        self.push(Object::Float(result))?;
                    } else if let (Object::Integer(l), Object::Float(r)) = (&left, &right) {
                        let l_f = *l as f64;
                        let result = match op {
                            Opcode::OpAdd => l_f + r,
                            Opcode::OpSub => l_f - r,
                            Opcode::OpMul => l_f * r,
                            Opcode::OpDiv => {
                                if *r == 0.0 { return Err("division by zero".to_string()); }
                                l_f / r
                            }
                            Opcode::OpModulo => {
                                if *r == 0.0 { return Err("division by zero".to_string()); }
                                l_f % r
                            }
                            _ => unreachable!(),
                        };
                        self.push(Object::Float(result))?;
                    } else if let (Object::String(l), Object::String(r)) = (&left, &right) {
                        if op == Opcode::OpAdd {
                            self.push(Object::String(format!("{}{}", l, r)))?;
                        } else {
                            return Err(format!("Unsupported operator for strings: {:?}", op));
                        }
                    } else if matches!(left, Object::String(_)) || matches!(right, Object::String(_)) {
                        if op == Opcode::OpAdd {
                            self.push(Object::String(format!("{}{}", left, right)))?;
                        } else {
                            return Err(format!("Unsupported operator for string concatenation: {:?}", op));
                        }
                    } else {
                        return Err("Unsupported types for binary operation".to_string());
                    }
                }
                Opcode::OpEqual | Opcode::OpNotEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let eq = left == right;
                    self.push(Object::Boolean(if op == Opcode::OpEqual { eq } else { !eq }))?;
                }
                Opcode::OpLessThan | Opcode::OpGreaterThan | Opcode::OpLessEqual | Opcode::OpGreaterEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let res = match (&left, &right) {
                        (Object::Integer(l), Object::Integer(r)) => match op {
                            Opcode::OpLessThan => l < r,
                            Opcode::OpGreaterThan => l > r,
                            Opcode::OpLessEqual => l <= r,
                            Opcode::OpGreaterEqual => l >= r,
                            _ => unreachable!(),
                        },
                        (Object::Float(l), Object::Float(r)) => match op {
                            Opcode::OpLessThan => l < r,
                            Opcode::OpGreaterThan => l > r,
                            Opcode::OpLessEqual => l <= r,
                            Opcode::OpGreaterEqual => l >= r,
                            _ => unreachable!(),
                        },
                        (Object::Integer(l), Object::Float(r)) => match op {
                            Opcode::OpLessThan => (*l as f64) < *r,
                            Opcode::OpGreaterThan => (*l as f64) > *r,
                            Opcode::OpLessEqual => (*l as f64) <= *r,
                            Opcode::OpGreaterEqual => (*l as f64) >= *r,
                            _ => unreachable!(),
                        },
                        (Object::Float(l), Object::Integer(r)) => match op {
                            Opcode::OpLessThan => *l < (*r as f64),
                            Opcode::OpGreaterThan => *l > (*r as f64),
                            Opcode::OpLessEqual => *l <= (*r as f64),
                            Opcode::OpGreaterEqual => *l >= (*r as f64),
                            _ => unreachable!(),
                        },
                        _ => return Err(format!("Cannot compare {:?} and {:?}", left, right)),
                    };
                    self.push(Object::Boolean(res))?;
                }
                Opcode::OpMinus => {
                    let right = self.pop()?;
                    match right {
                        Object::Integer(i) => self.push(Object::Integer(-i))?,
                        Object::Float(f) => self.push(Object::Float(-f))?,
                        _ => return Err(format!("Unsupported type for prefix -: {:?}", right)),
                    }
                }
                Opcode::OpBang => {
                    let right = self.pop()?;
                    let truthy = is_truthy(&right);
                    self.push(Object::Boolean(!truthy))?;
                }
                Opcode::OpPop => {
                    let popped = self.pop()?;
                    self.last_popped = Some(popped);
                }
                Opcode::OpTrue => {
                    self.push(Object::Boolean(true))?;
                }
                Opcode::OpFalse => {
                    self.push(Object::Boolean(false))?;
                }
                Opcode::OpNull => {
                    self.push(Object::Null)?;
                }
                Opcode::OpSetGlobal => {
                    let global_index = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    ip += 2;
                    let val = self.pop()?;
                    self.globals[global_index] = val.clone();
                    if global_index < self.symbol_names.len() {
                        let name = self.symbol_names[global_index].clone();
                        self.env.borrow_mut().set(name, val, true);
                    }
                }
                Opcode::OpGetGlobal => {
                    let global_index = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    ip += 2;
                    let val = if global_index < self.symbol_names.len() {
                        let name = &self.symbol_names[global_index];
                        if let Some(v) = self.env.borrow().get(name) {
                            self.globals[global_index] = v.clone();
                            v
                        } else {
                            self.globals[global_index].clone()
                        }
                    } else {
                        self.globals[global_index].clone()
                    };
                    self.push(val)?;
                }
                Opcode::OpJump => {
                    let target = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    ip = target;
                    continue;
                }
                Opcode::OpJumpNotTruthy => {
                    let target = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    let condition = self.pop()?;
                    if !is_truthy(&condition) {
                        ip = target;
                        continue;
                    } else {
                        ip += 2;
                    }
                }
                Opcode::OpRange => {
                    let inclusive = self.instructions[ip + 1] != 0;
                    ip += 1;
                    let end_obj = self.pop()?;
                    let start_obj = self.pop()?;
                    match (start_obj, end_obj) {
                        (Object::Integer(start), Object::Integer(end)) => {
                            self.push(Object::Range { start, end, inclusive })?;
                        }
                        _ => return Err("Range operands must be integers".to_string()),
                    }
                }
                Opcode::OpArray => {
                    let num_elements = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    ip += 2;
                    let mut elements = vec![Object::Null; num_elements];
                    for i in (0..num_elements).rev() {
                        elements[i] = self.pop()?;
                    }
                    self.push(Object::Array(Rc::new(RefCell::new(elements))))?;
                }
                Opcode::OpHash => {
                    let num_elements = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    ip += 2;
                    let num_pairs = num_elements / 2;
                    let mut entries = Vec::with_capacity(num_pairs);
                    for _ in 0..num_pairs {
                        let value = self.pop()?;
                        let key_obj = self.pop()?;
                        let hash_key = key_obj.get_hash_key().map_err(|e| e)?;
                        entries.push((hash_key, value));
                    }
                    let mut map = HashMap::new();
                    for (k, v) in entries.into_iter().rev() {
                        map.insert(k, v);
                    }
                    self.push(Object::Hash(Rc::new(RefCell::new(map))))?;
                }
                Opcode::OpIndex => {
                    let index = self.pop()?;
                    let container = self.pop()?;
                    match (container, index) {
                        (Object::Array(rc), Object::Integer(idx)) => {
                            let elements = rc.borrow();
                            if idx < 0 || idx as usize >= elements.len() {
                                self.push(Object::Null)?;
                            } else {
                                self.push(elements[idx as usize].clone())?;
                            }
                        }
                        (Object::Hash(rc), index_val) => {
                            let hash_key = index_val.get_hash_key().map_err(|e| e)?;
                            let map = rc.borrow();
                            if let Some(val) = map.get(&hash_key) {
                                self.push(val.clone())?;
                            } else {
                                self.push(Object::Null)?;
                            }
                        }
                        (Object::StructInstance { struct_name, fields }, Object::String(field_name)) => {
                            let map = fields.borrow();
                            if let Some(val) = map.get(&field_name) {
                                self.push(val.clone())?;
                            } else {
                                return Err(format!("field '{}' not found on struct '{}'", field_name, struct_name));
                            }
                        }
                        (c, _) => return Err(format!("Index operator not supported for {:?}", c)),
                    }
                }
                Opcode::OpSetIndex => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let container = self.pop()?;
                    match container {
                        Object::Array(rc) => {
                            let idx = match index {
                                Object::Integer(i) => i,
                                _ => return Err("index must be integer".to_string()),
                            };
                            let mut vec = rc.borrow_mut();
                            if idx >= 0 && (idx as usize) < vec.len() {
                                vec[idx as usize] = value;
                            } else if idx >= 0 && (idx as usize) == vec.len() {
                                vec.push(value);
                            } else {
                                return Err(format!("index out of bounds: {}", idx));
                            }
                        }
                        Object::Hash(rc) => {
                            let key = index.get_hash_key().map_err(|e| e)?;
                            rc.borrow_mut().insert(key, value);
                        }
                        Object::StructInstance { struct_name, fields } => {
                            let field_name = match index {
                                Object::String(s) => s,
                                _ => return Err("struct field must be string".to_string()),
                            };
                            let mut map = fields.borrow_mut();
                            if !map.contains_key(&field_name) {
                                return Err(format!("field '{}' not found on struct '{}'", field_name, struct_name));
                            }
                            map.insert(field_name, value);
                        }
                        _ => return Err("target is not indexable or mutable".to_string()),
                    }
                }
                Opcode::OpGetBuiltin => {
                    let builtin_index = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    ip += 2;
                    if builtin_index < crate::compiler::BUILTINS.len() {
                        let name = crate::compiler::BUILTINS[builtin_index];
                        self.push(Object::Builtin(name.to_string()))?;
                    } else {
                        return Err(format!("Builtin index out of bounds: {}", builtin_index));
                    }
                }
                Opcode::OpCall => {
                    let num_args = self.instructions[ip + 1] as usize;
                    ip += 1;
                    let mut args = vec![Object::Null; num_args];
                    for i in (0..num_args).rev() {
                        args[i] = self.pop()?;
                    }
                    let callee = self.pop()?;
                    match callee {
                        Object::Builtin(name) => {
                            let result = crate::evaluator::apply_builtin(&name, args);
                            if let Object::Error(err) = result {
                                return Err(err);
                            }
                            self.push(result)?;
                        }
                        Object::Function { .. } => {
                            let result = crate::evaluator::apply_function(callee, args);
                            if let Object::Error(err) = result {
                                return Err(err);
                            }
                            self.push(result)?;
                        }
                        Object::StructDef { name, fields } => {
                            if args.len() != fields.len() {
                                return Err(format!("struct '{}' expects {} arguments, got {}", name, fields.len(), args.len()));
                            }
                            let mut instance_fields = HashMap::new();
                            for (i, (f_name, f_type)) in fields.iter().enumerate() {
                                let arg = &args[i];
                                if let Some(expected_type) = f_type {
                                    let actual_type = arg.type_name();
                                    if actual_type != *expected_type && expected_type != "Any" {
                                        return Err(format!("type mismatch for struct field '{}': expected {}, got {}", f_name, expected_type, actual_type));
                                    }
                                }
                                instance_fields.insert(f_name.clone(), arg.clone());
                            }
                            self.push(Object::StructInstance {
                                struct_name: name,
                                fields: Rc::new(RefCell::new(instance_fields)),
                            })?;
                        }
                        _ => return Err(format!("Calling non-callable object in VM is not supported: {:?}", callee)),
                    }
                }
                Opcode::OpIterInit => {
                    let iterable = self.pop()?;
                    match iterable {
                        Object::Range { .. } | Object::Array(_) => {
                            self.push(Object::Iterator {
                                target: Box::new(iterable),
                                current: 0,
                            })?;
                        }
                        _ => return Err(format!("Cannot iterate over {:?}", iterable)),
                    }
                }
                Opcode::OpIterNext => {
                    let jump_target = ((self.instructions[ip + 1] as usize) << 8) | (self.instructions[ip + 2] as usize);
                    let mut iter_obj = self.stack[self.sp - 1].clone();
                    match &mut iter_obj {
                        Object::Iterator { target, current } => {
                            match target.as_ref() {
                                Object::Range { start, end, inclusive } => {
                                    let curr_val = match start.checked_add(*current) {
                                        Some(v) => v,
                                        None => {
                                            self.pop()?;
                                            ip = jump_target;
                                            continue;
                                        }
                                    };
                                    let has_next = if *inclusive { curr_val <= *end } else { curr_val < *end };
                                    if has_next {
                                        let next_elem = Object::Integer(curr_val);
                                        *current += 1;
                                        self.stack[self.sp - 1] = iter_obj;
                                        self.push(next_elem)?;
                                        ip += 2;
                                    } else {
                                        self.pop()?; // pop iterator
                                        ip = jump_target;
                                        continue;
                                    }
                                }
                                Object::Array(rc) => {
                                    let elements = rc.borrow();
                                    if (*current as usize) < elements.len() {
                                        let next_elem = elements[*current as usize].clone();
                                        drop(elements);
                                        *current += 1;
                                        self.stack[self.sp - 1] = iter_obj;
                                        self.push(next_elem)?;
                                        ip += 2;
                                    } else {
                                        self.pop()?; // pop iterator
                                        ip = jump_target;
                                        continue;
                                    }
                                }
                                _ => return Err("Invalid iterator target".to_string()),
                            }
                        }
                        _ => return Err("Expected iterator on stack for OpIterNext".to_string()),
                    }
                }
            }
            
            ip += 1;
        }
        Ok(())
    }

    fn push(&mut self, obj: Object) -> Result<(), String> {
        if self.sp >= STACK_SIZE {
            return Err("Stack overflow".to_string());
        }
        self.stack[self.sp] = obj;
        self.sp += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<Object, String> {
        if self.sp == 0 {
            return Err("Stack empty".to_string());
        }
        self.sp -= 1;
        Ok(self.stack[self.sp].clone())
    }
}
