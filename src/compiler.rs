use crate::ast::{Expression, Program, Statement};
use crate::code::{make, Opcode, Instructions};
use crate::object::Object;

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Vec::new(),
            constants: Vec::new(),
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
            Statement::Let { name, value, .. } => {
                // Just for primitive VM demo, map names to simple integer indices?
                // Real compiler needs symbol table. We'll skip variables in V0 to keep it simple.
                self.compile_expression(value)?;
            }
            _ => return Err("Unsupported statement in VM compilation".to_string()),
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
            Expression::Infix { left, operator, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                match operator.as_str() {
                    "+" => { self.emit(Opcode::OpAdd, &[]); }
                    "-" => { self.emit(Opcode::OpSub, &[]); }
                    "*" => { self.emit(Opcode::OpMul, &[]); }
                    "/" => { self.emit(Opcode::OpDiv, &[]); }
                    _ => return Err(format!("Unknown operator: {}", operator)),
                }
            }
            _ => return Err("Unsupported expression in VM compilation".to_string()),
        }
        Ok(())
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
