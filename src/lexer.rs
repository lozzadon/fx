use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,      // current position in input (points to current char)
    read_position: usize, // current reading position in input (after current char)
    ch: char,             // current char under examination
    pub line: usize,
    pub column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            column: 0,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.ch == '\n' {
            self.line += 1;
            self.column = 0;
        }

        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
        self.column += 1;
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::Equal
                } else if self.peek_char() == '>' {
                    self.read_char();
                    Token::FatArrow
                } else {
                    Token::Assign
                }
            }
            '+' => Token::Plus,
            '-' => {
                if self.peek_char() == '>' {
                    self.read_char();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NotEqual
                } else {
                    Token::Bang
                }
            }
            '&' => {
                if self.peek_char() == '&' {
                    self.read_char();
                    Token::And
                } else {
                    Token::Illegal(self.ch)
                }
            }
            '|' => {
                if self.peek_char() == '|' {
                    self.read_char();
                    Token::Or
                } else {
                    Token::Illegal(self.ch)
                }
            }
            ',' => Token::Comma,
            ':' => Token::Colon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '\0' => Token::Eof,
            '"' => self.read_string(),
            _ => {
                if self.is_letter(self.ch) {
                    let ident = self.read_identifier();
                    return Lexer::lookup_ident(&ident);
                } else if self.ch.is_ascii_digit() {
                    return self.read_number();
                } else {
                    Token::Illegal(self.ch)
                }
            }
        };

        self.read_char();
        token
    }

    fn read_string(&mut self) -> Token {
        self.read_char(); // skip the opening quote
        let start_pos = self.position;
        while self.ch != '"' && self.ch != '\0' {
            self.read_char();
        }
        let string_val: String = self.input[start_pos..self.position].iter().collect();
        Token::String(string_val)
    }

    fn read_identifier(&mut self) -> String {
        let start_pos = self.position;
        while self.is_letter(self.ch) || self.ch.is_ascii_digit() {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    fn read_number(&mut self) -> Token {
        let position = self.read_position - 1;
        let mut is_float = false;

        while self.ch.is_ascii_digit() || self.ch == '.' {
            if self.ch == '.' {
                is_float = true;
            }
            self.read_char();
        }

        let num_str: String = self.input[position..self.read_position - 1].iter().collect();
        
        if is_float {
            Token::Float(num_str.parse::<f64>().unwrap_or(0.0))
        } else {
            Token::Int(num_str.parse::<i64>().unwrap_or(0))
        }
    }

    fn is_letter(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
                self.read_char();
            } else if self.ch == '/' && self.peek_char() == '/' {
                // Skip comment
                while self.ch != '\n' && self.ch != '\0' {
                    self.read_char();
                }
            } else {
                break;
            }
        }
    }

    fn lookup_ident(ident: &str) -> Token {
        match ident {
            "func" => Token::Func,
            "let" => Token::Let,
            "var" => Token::Var,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "nil" => Token::Null,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "match" => Token::Match,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "throw" => Token::Throw,
            _ => Token::Ident(ident.to_string()),
        }
    }

    pub fn get_line(&self, line_num: usize) -> Option<String> {
        let mut current_line = 1;
        let mut start_idx = 0;
        
        for (i, &c) in self.input.iter().enumerate() {
            if current_line == line_num {
                if c == '\n' {
                    return Some(self.input[start_idx..i].iter().collect());
                }
            } else if c == '\n' {
                current_line += 1;
                start_idx = i + 1;
            }
        }
        
        if current_line == line_num && start_idx < self.input.len() {
            return Some(self.input[start_idx..].iter().collect());
        }
        
        None
    }
}
