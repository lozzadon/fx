use crate::ast::{Expression, Program, Statement};
use crate::code::{make, Opcode, Instructions};
use crate::object::{Object, Environment};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
    pub symbol_names: Vec<String>,
    pub symbol_mutability: Vec<bool>,
    pub symbol_is_global: Vec<bool>,
    pub env: Rc<RefCell<Environment>>,
}

pub const BUILTINS: &[&str] = &[
    "len", "push", "pop", "print", "map", "filter", "reduce", "import",
    "split", "trim", "replace", "join", "contains", "starts_with", "ends_with",
    "to_upper", "to_lower", "substring",
];

pub fn lookup_builtin(name: &str) -> Option<usize> {
    BUILTINS.iter().position(|&b| b == name)
}

#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    pub index: usize,
    pub is_mutable: bool,
}

pub struct GlobalSymbolState {
    pub num_definitions: usize,
    pub symbol_names: Vec<String>,
    pub symbol_mutability: Vec<bool>,
    pub symbol_is_global: Vec<bool>,
}

pub struct SymbolTable {
    pub store: HashMap<String, Symbol>,
    pub outer: Option<Box<SymbolTable>>,
    pub global_state: Rc<RefCell<GlobalSymbolState>>,
    pub depth: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            store: HashMap::new(),
            outer: None,
            global_state: Rc::new(RefCell::new(GlobalSymbolState {
                num_definitions: 0,
                symbol_names: Vec::new(),
                symbol_mutability: Vec::new(),
                symbol_is_global: Vec::new(),
            })),
            depth: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        let current_store = std::mem::take(&mut self.store);
        let current_outer = self.outer.take();
        let old_self = SymbolTable {
            store: current_store,
            outer: current_outer,
            global_state: Rc::clone(&self.global_state),
            depth: self.depth,
        };
        self.outer = Some(Box::new(old_self));
        self.depth += 1;
    }

    pub fn leave_scope(&mut self) {
        if let Some(outer) = self.outer.take() {
            self.store = outer.store;
            self.depth = outer.depth;
            self.outer = outer.outer;
        }
    }
    
    pub fn define(&mut self, name: String, is_mutable: bool) -> usize {
        if let Some(sym) = self.store.get_mut(&name) {
            sym.is_mutable = is_mutable;
            let mut state = self.global_state.borrow_mut();
            state.symbol_mutability[sym.index] = is_mutable;
            sym.index
        } else {
            let mut state = self.global_state.borrow_mut();
            let index = state.num_definitions;
            self.store.insert(name.clone(), Symbol { index, is_mutable });
            state.symbol_names.push(name);
            state.symbol_mutability.push(is_mutable);
            state.symbol_is_global.push(self.depth == 0);
            state.num_definitions += 1;
            index
        }
    }
    
    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        if let Some(sym) = self.store.get(name) {
            Some(sym.clone())
        } else if let Some(outer) = &self.outer {
            outer.resolve(name)
        } else {
            None
        }
    }
}

pub struct LoopContext {
    pub loop_start: usize,
    pub break_positions: Vec<usize>,
    pub requires_iterator_pop: bool,
}

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
    pub symbol_table: SymbolTable,
    pub env: Rc<RefCell<Environment>>,
    pub loop_stack: Vec<LoopContext>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Vec::new(),
            constants: Vec::new(),
            symbol_table: SymbolTable::new(),
            env: Rc::new(RefCell::new(Environment::new())),
            loop_stack: Vec::new(),
        }
    }

    pub fn compile(&mut self, program: &Program) -> Result<(), String> {
        for s in &program.statements {
            self.compile_statement(s)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, s: &Statement) -> Result<(), String> {
        match s {
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::OpPop, &[]);
            }
            Statement::Let { name, value, is_mutable } => {
                self.compile_expression(value)?;
                let symbol_index = self.symbol_table.define(name.clone(), *is_mutable);
                self.env.borrow_mut().set(name.clone(), Object::Null, *is_mutable);
                self.emit(Opcode::OpSetGlobal, &[symbol_index]);
            }
            Statement::Assign { name, value } => {
                self.compile_expression(value)?;
                if let Some(symbol) = self.symbol_table.resolve(name) {
                    if !symbol.is_mutable {
                        return Err(format!("cannot assign to immutable variable '{}'", name));
                    }
                    self.emit(Opcode::OpSetGlobal, &[symbol.index]);
                } else {
                    return Err(format!("cannot assign to undefined variable {}", name));
                }
            }
            Statement::IndexAssign { left, index, value } => {
                self.compile_expression(left)?;
                self.compile_expression(index)?;
                self.compile_expression(value)?;
                self.emit(Opcode::OpSetIndex, &[]);
            }
            Statement::FieldAssign { object, field, value } => {
                self.compile_expression(object)?;
                let field_const = self.add_constant(Object::String(field.clone()));
                self.emit(Opcode::OpConstant, &[field_const]);
                self.compile_expression(value)?;
                self.emit(Opcode::OpSetIndex, &[]);
            }
            Statement::StructDef { name, fields } => {
                let struct_obj = Object::StructDef {
                    name: name.clone(),
                    fields: fields.clone(),
                };
                let const_idx = self.add_constant(struct_obj);
                self.emit(Opcode::OpConstant, &[const_idx]);
                let symbol_index = self.symbol_table.define(name.clone(), false);
                self.emit(Opcode::OpSetGlobal, &[symbol_index]);
            }
            Statement::Block(statements) => {
                self.symbol_table.enter_scope();
                for stmt in statements {
                    if let Err(e) = self.compile_statement(stmt) {
                        self.symbol_table.leave_scope();
                        return Err(e);
                    }
                }
                self.symbol_table.leave_scope();
            }
            Statement::Break => {
                if self.loop_stack.is_empty() {
                    return Err("break statement not within a loop".to_string());
                }
                if self.loop_stack.last().unwrap().requires_iterator_pop {
                    self.emit(Opcode::OpPop, &[]);
                }
                let pos = self.emit(Opcode::OpJump, &[0xFFFF]);
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.break_positions.push(pos);
                }
            }
            Statement::Continue => {
                if self.loop_stack.is_empty() {
                    return Err("continue statement not within a loop".to_string());
                }
                let target = self.loop_stack.last().unwrap().loop_start;
                self.emit(Opcode::OpJump, &[target]);
            }
            _ => return Err(format!("Unsupported statement in VM compilation: {:?}", s)),
        }
        Ok(())
    }

    fn compile_block_expression(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Block(statements) => {
                self.symbol_table.enter_scope();
                if statements.is_empty() {
                    self.emit(Opcode::OpNull, &[]);
                    self.symbol_table.leave_scope();
                    return Ok(());
                }
                for (i, s) in statements.iter().enumerate() {
                    if i == statements.len() - 1 {
                        if let Statement::Expression(expr) = s {
                            if let Err(e) = self.compile_expression(expr) {
                                self.symbol_table.leave_scope();
                                return Err(e);
                            }
                        } else {
                            if let Err(e) = self.compile_statement(s) {
                                self.symbol_table.leave_scope();
                                return Err(e);
                            }
                            self.emit(Opcode::OpNull, &[]);
                        }
                    } else {
                        if let Err(e) = self.compile_statement(s) {
                            self.symbol_table.leave_scope();
                            return Err(e);
                        }
                    }
                }
                self.symbol_table.leave_scope();
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
            }
            _ => {
                self.compile_statement(stmt)?;
                self.emit(Opcode::OpNull, &[]);
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, e: &Expression) -> Result<(), String> {
        match e {
            Expression::IntegerLiteral(i) => {
                let obj = Object::Integer(*i);
                let pos = self.add_constant(obj);
                self.emit(Opcode::OpConstant, &[pos]);
            }
            Expression::FloatLiteral(f) => {
                let obj = Object::Float(*f);
                let pos = self.add_constant(obj);
                self.emit(Opcode::OpConstant, &[pos]);
            }
            Expression::Boolean(b) => {
                if *b {
                    self.emit(Opcode::OpTrue, &[]);
                } else {
                    self.emit(Opcode::OpFalse, &[]);
                }
            }
            Expression::StringLiteral(s) => {
                let obj = Object::String(s.clone());
                let pos = self.add_constant(obj);
                self.emit(Opcode::OpConstant, &[pos]);
            }
            Expression::NullLiteral => {
                self.emit(Opcode::OpNull, &[]);
            }
            Expression::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.resolve(name) {
                    self.emit(Opcode::OpGetGlobal, &[symbol.index]);
                } else if let Some(builtin_idx) = lookup_builtin(name) {
                    self.emit(Opcode::OpGetBuiltin, &[builtin_idx]);
                } else {
                    return Err(format!("undefined variable {}", name));
                }
            }
            Expression::Prefix { operator, right } => {
                self.compile_expression(right)?;
                match operator.as_str() {
                    "-" => { self.emit(Opcode::OpMinus, &[]); }
                    "!" => { self.emit(Opcode::OpBang, &[]); }
                    _ => return Err(format!("Unknown prefix operator: {}", operator)),
                }
            }
            Expression::Infix { left, operator, right } => {
                if operator == "&&" {
                    self.compile_expression(left)?;
                    let jump_not_truthy = self.emit(Opcode::OpJumpNotTruthy, &[0xFFFF]);
                    self.compile_expression(right)?;
                    let jump_end = self.emit(Opcode::OpJump, &[0xFFFF]);
                    let false_pos = self.instructions.len();
                    self.change_operand(jump_not_truthy, false_pos);
                    self.emit(Opcode::OpFalse, &[]);
                    let end_pos = self.instructions.len();
                    self.change_operand(jump_end, end_pos);
                    return Ok(());
                }
                if operator == "||" {
                    self.compile_expression(left)?;
                    let jump_not_truthy = self.emit(Opcode::OpJumpNotTruthy, &[0xFFFF]);
                    self.emit(Opcode::OpTrue, &[]);
                    let jump_end = self.emit(Opcode::OpJump, &[0xFFFF]);
                    let eval_right_pos = self.instructions.len();
                    self.change_operand(jump_not_truthy, eval_right_pos);
                    self.compile_expression(right)?;
                    let end_pos = self.instructions.len();
                    self.change_operand(jump_end, end_pos);
                    return Ok(());
                }

                self.compile_expression(left)?;
                self.compile_expression(right)?;
                match operator.as_str() {
                    "+" => { self.emit(Opcode::OpAdd, &[]); }
                    "-" => { self.emit(Opcode::OpSub, &[]); }
                    "*" => { self.emit(Opcode::OpMul, &[]); }
                    "/" => { self.emit(Opcode::OpDiv, &[]); }
                    "%" => { self.emit(Opcode::OpModulo, &[]); }
                    "==" => { self.emit(Opcode::OpEqual, &[]); }
                    "!=" => { self.emit(Opcode::OpNotEqual, &[]); }
                    "<" => { self.emit(Opcode::OpLessThan, &[]); }
                    ">" => { self.emit(Opcode::OpGreaterThan, &[]); }
                    "<=" => { self.emit(Opcode::OpLessEqual, &[]); }
                    ">=" => { self.emit(Opcode::OpGreaterEqual, &[]); }
                    _ => return Err(format!("Unknown operator: {}", operator)),
                }
            }
            Expression::Range { start, end, inclusive } => {
                self.compile_expression(start)?;
                self.compile_expression(end)?;
                self.emit(Opcode::OpRange, &[if *inclusive { 1 } else { 0 }]);
            }
            Expression::Array(elements) => {
                for el in elements {
                    self.compile_expression(el)?;
                }
                self.emit(Opcode::OpArray, &[elements.len()]);
            }
            Expression::HashLiteral(pairs) => {
                for (k, v) in pairs {
                    self.compile_expression(k)?;
                    self.compile_expression(v)?;
                }
                self.emit(Opcode::OpHash, &[pairs.len() * 2]);
            }
            Expression::FieldAccess { object, field } => {
                self.compile_expression(object)?;
                let field_const = self.add_constant(Object::String(field.clone()));
                self.emit(Opcode::OpConstant, &[field_const]);
                self.emit(Opcode::OpIndex, &[]);
            }
            Expression::Index { left, index } => {
                self.compile_expression(left)?;
                self.compile_expression(index)?;
                self.emit(Opcode::OpIndex, &[]);
            }
            Expression::Call { function, arguments } => {
                self.compile_expression(function)?;
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                self.emit(Opcode::OpCall, &[arguments.len()]);
            }
            Expression::If { condition, consequence, alternative } => {
                self.compile_expression(condition)?;
                let jump_not_truthy_pos = self.emit(Opcode::OpJumpNotTruthy, &[0xFFFF]);
                self.compile_block_expression(consequence)?;
                let jump_pos = self.emit(Opcode::OpJump, &[0xFFFF]);
                let alt_pos = self.instructions.len();
                self.change_operand(jump_not_truthy_pos, alt_pos);
                if let Some(alt) = alternative {
                    self.compile_block_expression(alt)?;
                } else {
                    self.emit(Opcode::OpNull, &[]);
                }
                let after_if = self.instructions.len();
                self.change_operand(jump_pos, after_if);
            }
            Expression::While { condition, body } => {
                let loop_start = self.instructions.len();
                self.compile_expression(condition)?;
                let jump_not_truthy_pos = self.emit(Opcode::OpJumpNotTruthy, &[0xFFFF]);
                
                self.loop_stack.push(LoopContext {
                    loop_start,
                    break_positions: Vec::new(),
                    requires_iterator_pop: false,
                });
                
                self.compile_statement(body)?;
                self.emit(Opcode::OpJump, &[loop_start]);
                let after_loop = self.instructions.len();
                self.change_operand(jump_not_truthy_pos, after_loop);
                
                let ctx = self.loop_stack.pop().unwrap();
                for pos in ctx.break_positions {
                    self.change_operand(pos, after_loop);
                }
                
                self.emit(Opcode::OpNull, &[]);
            }
            Expression::For { variable, iterable, body } => {
                self.compile_expression(iterable)?;
                self.emit(Opcode::OpIterInit, &[]);
                let loop_start = self.instructions.len();
                let iter_next_pos = self.emit(Opcode::OpIterNext, &[0xFFFF]);
                
                self.symbol_table.enter_scope();
                let symbol_index = self.symbol_table.define(variable.clone(), false);
                self.emit(Opcode::OpSetGlobal, &[symbol_index]);
                
                self.loop_stack.push(LoopContext {
                    loop_start,
                    break_positions: Vec::new(),
                    requires_iterator_pop: true,
                });
                
                if let Err(e) = self.compile_statement(body) {
                    self.symbol_table.leave_scope();
                    return Err(e);
                }
                
                self.emit(Opcode::OpJump, &[loop_start]);
                let after_loop = self.instructions.len();
                self.change_operand(iter_next_pos, after_loop);
                
                let ctx = self.loop_stack.pop().unwrap();
                for pos in ctx.break_positions {
                    self.change_operand(pos, after_loop);
                }
                
                self.emit(Opcode::OpNull, &[]);
                self.symbol_table.leave_scope();
            }
            Expression::FunctionLiteral { parameters, return_type, body, .. } => {
                let func_obj = Object::Function {
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: *body.clone(),
                    env: Rc::clone(&self.env),
                };
                let const_idx = self.add_constant(func_obj);
                self.emit(Opcode::OpConstant, &[const_idx]);
            }
            Expression::Match { value, cases } => {
                self.compile_expression(value)?;
                let mut end_jumps = Vec::new();

                for (pattern, body) in cases {
                    let is_catch_all = if let Expression::Identifier(name) = &pattern { name == "_" } else { false };
                    
                    if is_catch_all {
                        self.emit(Opcode::OpPop, &[]);
                        self.compile_block_expression(body)?;
                        let jmp = self.emit(Opcode::OpJump, &[0xFFFF]);
                        end_jumps.push(jmp);
                        break;
                    }

                    self.emit(Opcode::OpDup, &[]);
                    self.compile_expression(pattern)?;
                    self.emit(Opcode::OpEqual, &[]);
                    
                    let jump_not_match = self.emit(Opcode::OpJumpNotTruthy, &[0xFFFF]);
                    
                    self.emit(Opcode::OpPop, &[]); // Pop the value on match
                    self.compile_block_expression(body)?;
                    let jmp_end = self.emit(Opcode::OpJump, &[0xFFFF]);
                    end_jumps.push(jmp_end);
                    
                    let next_case_pos = self.instructions.len();
                    self.change_operand(jump_not_match, next_case_pos);
                }

                self.emit(Opcode::OpPop, &[]); // Pop the value if no match
                self.emit(Opcode::OpNull, &[]);
                let default_end = self.emit(Opcode::OpJump, &[0xFFFF]);
                end_jumps.push(default_end);

                let end_pos = self.instructions.len();
                for jmp in end_jumps {
                    self.change_operand(jmp, end_pos);
                }
            }
            Expression::TryCatch { try_body, catch_param, catch_body } => {
                let try_jump = self.emit(Opcode::OpTry, &[0xFFFF]);
                
                self.compile_block_expression(try_body)?;
                self.emit(Opcode::OpCatchEnd, &[]);
                let end_jump = self.emit(Opcode::OpJump, &[0xFFFF]);
                
                let catch_pos = self.instructions.len();
                self.change_operand(try_jump, catch_pos);
                
                self.symbol_table.enter_scope();
                let catch_param_idx = self.symbol_table.define(catch_param.clone(), false);
                self.emit(Opcode::OpSetGlobal, &[catch_param_idx]);
                
                self.compile_block_expression(catch_body)?;
                
                self.symbol_table.leave_scope();
                
                let end_pos = self.instructions.len();
                self.change_operand(end_jump, end_pos);
            }
            Expression::Throw(exp) => {
                self.compile_expression(exp)?;
                self.emit(Opcode::OpThrow, &[]);
            }
        }
        Ok(())
    }

    pub fn change_operand(&mut self, op_pos: usize, operand: usize) {
        self.instructions[op_pos + 1] = (operand >> 8) as u8;
        self.instructions[op_pos + 2] = operand as u8;
    }

    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn emit(&mut self, op: Opcode, operands: &[usize]) -> usize {
        let ins = make(op, operands);
        let pos = self.instructions.len();
        self.instructions.extend(ins);
        pos
    }

    pub fn bytecode(self) -> Bytecode {
        let global_state = self.symbol_table.global_state.borrow();
        Bytecode {
            instructions: self.instructions,
            constants: self.constants,
            symbol_names: global_state.symbol_names.clone(),
            symbol_mutability: global_state.symbol_mutability.clone(),
            symbol_is_global: global_state.symbol_is_global.clone(),
            env: self.env,
        }
    }
}
