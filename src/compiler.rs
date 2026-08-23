use crate::ast::{Expression, Program, Statement};
use crate::code::{make, Opcode, Instructions};
use crate::object::Object;
use std::collections::HashMap;

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
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

pub struct SymbolTable {
    pub store: HashMap<String, Symbol>,
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            store: HashMap::new(),
            num_definitions: 0,
        }
    }
    
    pub fn define(&mut self, name: String, is_mutable: bool) -> usize {
        if let Some(sym) = self.store.get_mut(&name) {
            sym.is_mutable = is_mutable;
            sym.index
        } else {
            let index = self.num_definitions;
            self.store.insert(name, Symbol { index, is_mutable });
            self.num_definitions += 1;
            index
        }
    }
    
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.store.get(name)
    }
}

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
    pub symbol_table: SymbolTable,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Vec::new(),
            constants: Vec::new(),
            symbol_table: SymbolTable::new(),
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
            Statement::Block(statements) => {
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
            }
            _ => return Err(format!("Unsupported statement in VM compilation: {:?}", s)),
        }
        Ok(())
    }

    fn compile_block_expression(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Block(statements) => {
                if statements.is_empty() {
                    self.emit(Opcode::OpNull, &[]);
                    return Ok(());
                }
                for (i, s) in statements.iter().enumerate() {
                    if i == statements.len() - 1 {
                        if let Statement::Expression(expr) = s {
                            self.compile_expression(expr)?;
                        } else {
                            self.compile_statement(s)?;
                            self.emit(Opcode::OpNull, &[]);
                        }
                    } else {
                        self.compile_statement(s)?;
                    }
                }
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
                if let Some(builtin_idx) = lookup_builtin(name) {
                    self.emit(Opcode::OpGetBuiltin, &[builtin_idx]);
                } else if let Some(symbol) = self.symbol_table.resolve(name) {
                    self.emit(Opcode::OpGetGlobal, &[symbol.index]);
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
                self.compile_statement(body)?;
                self.emit(Opcode::OpJump, &[loop_start]);
                let after_loop = self.instructions.len();
                self.change_operand(jump_not_truthy_pos, after_loop);
                self.emit(Opcode::OpNull, &[]);
            }
            Expression::For { variable, iterable, body } => {
                self.compile_expression(iterable)?;
                self.emit(Opcode::OpIterInit, &[]);
                let loop_start = self.instructions.len();
                let iter_next_pos = self.emit(Opcode::OpIterNext, &[0xFFFF]);
                let prev_symbol = self.symbol_table.resolve(variable).cloned();
                let symbol_index = self.symbol_table.define(variable.clone(), false);
                self.emit(Opcode::OpSetGlobal, &[symbol_index]);
                self.compile_statement(body)?;
                self.emit(Opcode::OpJump, &[loop_start]);
                let after_loop = self.instructions.len();
                self.change_operand(iter_next_pos, after_loop);
                self.emit(Opcode::OpNull, &[]);
                if let Some(prev) = prev_symbol {
                    if let Some(sym) = self.symbol_table.store.get_mut(variable) {
                        sym.is_mutable = prev.is_mutable;
                    }
                } else {
                    self.symbol_table.store.remove(variable);
                }
            }
            _ => return Err(format!("Unsupported expression in VM compilation: {:?}", e)),
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
        Bytecode {
            instructions: self.instructions,
            constants: self.constants,
        }
    }
}
