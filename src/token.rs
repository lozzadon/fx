#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Identifiers and Literals
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),

    // Keywords
    Func,   // func
    Let,    // let
    Var,    // var
    If,     // if
    Else,   // else
    Return, // return
    True,   // true
    False,  // false
    Null,   // null
    While,  // while
    For,    // for
    In,     // in
    Match,  // match
    Try,    // try
    Catch,  // catch
    Throw,  // throw
    Break,  // break
    Continue, // continue
    Struct, // struct

    // Operators
    Assign,          // =
    Plus,            // +
    Minus,           // -
    Asterisk,        // *
    Slash,           // /
    Percent,         // %
    PlusAssign,      // +=
    MinusAssign,     // -=
    AsteriskAssign,  // *=
    SlashAssign,     // /=
    PercentAssign,   // %=
    Equal,           // ==
    NotEqual,        // !=
    LessThan,        // <
    GreaterThan,     // >
    LessEqual,       // <=
    GreaterEqual,    // >=
    FatArrow,        // =>
    Arrow,           // ->
    Bang,            // !
    And,             // &&
    Or,              // ||
    DotDot,          // ..
    DotDotEqual,     // ..=
    Dot,             // .

    // Delimiters
    Comma,           // ,
    Colon,           // :
    LParen,          // (
    RParen,          // )
    LBrace,          // {
    RBrace,          // }
    LBracket,        // [
    RBracket,        // ]

    // Special
    Illegal(char),
    Eof,
}
