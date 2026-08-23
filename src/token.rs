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

    // Operators
    Assign,      // =
    Plus,        // +
    Minus,       // -
    Asterisk,    // *
    Slash,       // /
    Equal,       // ==
    NotEqual,    // !=
    LessThan,    // <
    GreaterThan, // >
    FatArrow,    // =>
    Arrow,       // ->
    Bang,        // !
    And,         // &&
    Or,          // ||

    // Delimiters
    Comma,       // ,
    Colon,       // :
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]

    // Special
    Illegal(char),
    Eof,
}
