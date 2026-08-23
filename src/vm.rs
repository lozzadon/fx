use crate::compiler::Bytecode;
use crate::object::Object;
use crate::code::{Opcode, Instructions};

const STACK_SIZE: usize = 2048;

pub struct VM {
    constants: Vec<Object>,
    instructions: Instructions,
    stack: Vec<Object>,
    sp: usize, // Stack pointer
    pub last_popped: Option<Object>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> VM {
        VM {
            constants: bytecode.constants,
            instructions: bytecode.instructions,
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
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
                    
                    self.push(self.constants[const_index].clone())?;
                }
                Opcode::OpAdd | Opcode::OpSub | Opcode::OpMul | Opcode::OpDiv => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    
                    if let (Object::Integer(l), Object::Integer(r)) = (left, right) {
                        let result = match op {
                            Opcode::OpAdd => l + r,
                            Opcode::OpSub => l - r,
                            Opcode::OpMul => l * r,
                            Opcode::OpDiv => l / r,
                            _ => unreachable!(),
                        };
                        self.push(Object::Integer(result))?;
                    } else {
                        return Err("Unsupported types for binary operation".to_string());
                    }
                }
                Opcode::OpPop => {
                    let popped = self.pop()?;
                    self.last_popped = Some(popped);
                }
                _ => return Err(format!("Opcode not implemented: {:?}", op)),
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
