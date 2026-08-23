#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    OpConstant,
    OpAdd,
    OpPop,
    OpSub,
    OpMul,
    OpDiv,
    OpTrue,
    OpFalse,
    OpEqual,
    OpNotEqual,
    OpGreaterThan,
    OpMinus,
    OpBang,
    OpJumpNotTruthy,
    OpJump,
    OpNull,
    OpSetGlobal,
    OpGetGlobal,
    OpLessThan,
    OpLessEqual,
    OpGreaterEqual,
    OpModulo,
    OpRange,
    OpArray,
    OpIndex,
    OpSetIndex,
    OpGetBuiltin,
    OpCall,
    OpIterInit,
    OpIterNext,
    OpHash,
}

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::OpConstant,
            1 => Opcode::OpAdd,
            2 => Opcode::OpPop,
            3 => Opcode::OpSub,
            4 => Opcode::OpMul,
            5 => Opcode::OpDiv,
            6 => Opcode::OpTrue,
            7 => Opcode::OpFalse,
            8 => Opcode::OpEqual,
            9 => Opcode::OpNotEqual,
            10 => Opcode::OpGreaterThan,
            11 => Opcode::OpMinus,
            12 => Opcode::OpBang,
            13 => Opcode::OpJumpNotTruthy,
            14 => Opcode::OpJump,
            15 => Opcode::OpNull,
            16 => Opcode::OpSetGlobal,
            17 => Opcode::OpGetGlobal,
            18 => Opcode::OpLessThan,
            19 => Opcode::OpLessEqual,
            20 => Opcode::OpGreaterEqual,
            21 => Opcode::OpModulo,
            22 => Opcode::OpRange,
            23 => Opcode::OpArray,
            24 => Opcode::OpIndex,
            25 => Opcode::OpSetIndex,
            26 => Opcode::OpGetBuiltin,
            27 => Opcode::OpCall,
            28 => Opcode::OpIterInit,
            29 => Opcode::OpIterNext,
            30 => Opcode::OpHash,
            _ => panic!("Unknown opcode: {}", v),
        }
    }
}

pub struct Definition {
    #[allow(dead_code)]
    pub name: &'static str,
    pub operand_widths: Vec<usize>,
}

pub fn lookup(op: u8) -> Definition {
    match Opcode::from(op) {
        Opcode::OpConstant => Definition { name: "OpConstant", operand_widths: vec![2] },
        Opcode::OpAdd => Definition { name: "OpAdd", operand_widths: vec![] },
        Opcode::OpPop => Definition { name: "OpPop", operand_widths: vec![] },
        Opcode::OpSub => Definition { name: "OpSub", operand_widths: vec![] },
        Opcode::OpMul => Definition { name: "OpMul", operand_widths: vec![] },
        Opcode::OpDiv => Definition { name: "OpDiv", operand_widths: vec![] },
        Opcode::OpTrue => Definition { name: "OpTrue", operand_widths: vec![] },
        Opcode::OpFalse => Definition { name: "OpFalse", operand_widths: vec![] },
        Opcode::OpEqual => Definition { name: "OpEqual", operand_widths: vec![] },
        Opcode::OpNotEqual => Definition { name: "OpNotEqual", operand_widths: vec![] },
        Opcode::OpGreaterThan => Definition { name: "OpGreaterThan", operand_widths: vec![] },
        Opcode::OpMinus => Definition { name: "OpMinus", operand_widths: vec![] },
        Opcode::OpBang => Definition { name: "OpBang", operand_widths: vec![] },
        Opcode::OpJumpNotTruthy => Definition { name: "OpJumpNotTruthy", operand_widths: vec![2] },
        Opcode::OpJump => Definition { name: "OpJump", operand_widths: vec![2] },
        Opcode::OpNull => Definition { name: "OpNull", operand_widths: vec![] },
        Opcode::OpSetGlobal => Definition { name: "OpSetGlobal", operand_widths: vec![2] },
        Opcode::OpGetGlobal => Definition { name: "OpGetGlobal", operand_widths: vec![2] },
        Opcode::OpLessThan => Definition { name: "OpLessThan", operand_widths: vec![] },
        Opcode::OpLessEqual => Definition { name: "OpLessEqual", operand_widths: vec![] },
        Opcode::OpGreaterEqual => Definition { name: "OpGreaterEqual", operand_widths: vec![] },
        Opcode::OpModulo => Definition { name: "OpModulo", operand_widths: vec![] },
        Opcode::OpRange => Definition { name: "OpRange", operand_widths: vec![1] },
        Opcode::OpArray => Definition { name: "OpArray", operand_widths: vec![2] },
        Opcode::OpIndex => Definition { name: "OpIndex", operand_widths: vec![] },
        Opcode::OpSetIndex => Definition { name: "OpSetIndex", operand_widths: vec![] },
        Opcode::OpGetBuiltin => Definition { name: "OpGetBuiltin", operand_widths: vec![2] },
        Opcode::OpCall => Definition { name: "OpCall", operand_widths: vec![1] },
        Opcode::OpIterInit => Definition { name: "OpIterInit", operand_widths: vec![] },
        Opcode::OpIterNext => Definition { name: "OpIterNext", operand_widths: vec![2] },
        Opcode::OpHash => Definition { name: "OpHash", operand_widths: vec![2] },
    }
}

pub type Instructions = Vec<u8>;

pub fn make(op: Opcode, operands: &[usize]) -> Vec<u8> {
    let def = lookup(op as u8);
    
    let mut instruction_len = 1;
    for w in &def.operand_widths {
        instruction_len += w;
    }
    
    let mut instruction = vec![0; instruction_len];
    instruction[0] = op as u8;
    
    let mut offset = 1;
    for (i, o) in operands.iter().enumerate() {
        let width = def.operand_widths[i];
        if width == 2 {
            instruction[offset] = (*o >> 8) as u8;
            instruction[offset + 1] = *o as u8;
        } else if width == 1 {
            instruction[offset] = *o as u8;
        }
        offset += width;
    }
    
    instruction
}
