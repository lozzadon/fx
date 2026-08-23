#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
        is_mutable: bool,
    },
    Assign {
        name: String,
        value: Expression,
    },
    IndexAssign {
        left: Expression,
        index: Expression,
        value: Expression,
    },
    FieldAssign {
        object: Expression,
        field: String,
        value: Expression,
    },
    StructDef {
        name: String,
        fields: Vec<(String, Option<String>)>, // (field_name, field_type)
    },
    Return(Expression),
    Expression(Expression),
    Block(Vec<Statement>), 
    Break,
    Continue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Boolean(bool),
    Prefix {
        operator: String,
        right: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        consequence: Box<Statement>, 
        alternative: Option<Box<Statement>>, 
    },
    While {
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    For {
        variable: String,
        iterable: Box<Expression>,
        body: Box<Statement>,
    },
    FunctionLiteral {
        name: Option<String>,
        parameters: Vec<(String, Option<String>)>, // (name, type_annotation)
        return_type: Option<String>,
        body: Box<Statement>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    StringLiteral(String),
    Array(Vec<Expression>),
    Index {
        left: Box<Expression>, // The array or hash
        index: Box<Expression>, // The index inside [ ]
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    NullLiteral,
    HashLiteral(Vec<(Expression, Expression)>),
    Match {
        value: Box<Expression>,
        cases: Vec<(Expression, Box<Statement>)>, // (pattern, consequence)
    },
    TryCatch {
        try_body: Box<Statement>,
        catch_param: String,
        catch_body: Box<Statement>,
    },
    Throw(Box<Expression>),
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
