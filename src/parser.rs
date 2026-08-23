use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Logical,     // &&, ||
    Equals,      // ==, !=
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        // myFunction(X)
    Index,       // array[index]
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Or | Token::And => Precedence::Logical,
        Token::Equal | Token::NotEqual => Precedence::Equals,
        Token::LessThan | Token::GreaterThan => Precedence::LessGreater,
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Asterisk | Token::Slash => Precedence::Product,
        Token::LParen => Precedence::Call,
        Token::LBracket => Precedence::Index,
        _ => Precedence::Lowest,
    }
}

pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
    pub errors: Vec<String>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Parser {
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            cur_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn cur_precedence(&self) -> Precedence {
        token_precedence(&self.cur_token)
    }

    fn peek_precedence(&self) -> Precedence {
        token_precedence(&self.peek_token)
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.cur_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }
        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token {
            Token::Let | Token::Var => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Func => self.parse_func_statement(),
            Token::Ident(_) if self.peek_token == Token::Assign => self.parse_assign_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_assign_statement(&mut self) -> Option<Statement> {
        let name = match &self.cur_token {
            Token::Ident(name) => name.clone(),
            _ => return None,
        };
        self.next_token(); // Move to '='
        self.next_token(); // Move to expression

        let value = self.parse_expression(Precedence::Lowest)?;

        Some(Statement::Assign { name, value })
    }

    fn parse_func_statement(&mut self) -> Option<Statement> {
        self.next_token();
        
        let name = match &self.cur_token {
            Token::Ident(name) => name.clone(),
            _ => {
                self.errors.push("Expected function name".to_string());
                return None;
            }
        };
        self.next_token(); 
        
        let parameters = self.parse_function_parameters()?;
        
        if self.peek_token == Token::Arrow {
            self.next_token(); 
            self.next_token(); 
        }
        
        if self.peek_token != Token::LBrace {
            self.errors.push("Expected { after function parameters".to_string());
            return None;
        }
        self.next_token(); 
        
        let body = self.parse_block_statement();
        
        let function_literal = Expression::FunctionLiteral {
            name: Some(name.clone()),
            parameters,
            body: Box::new(body),
        };

        Some(Statement::Let {
            name,
            value: function_literal,
            is_mutable: false, // Functions are immutable by default
        })
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<String>> {
        let mut parameters = Vec::new();
        
        if self.peek_token == Token::RParen {
            self.next_token(); 
            return Some(parameters);
        }
        self.next_token(); 
        
        if let Token::Ident(name) = &self.cur_token {
            parameters.push(name.clone());
        }
        
        if self.peek_token == Token::Colon {
            self.next_token(); 
            self.next_token(); 
        }

        while self.peek_token == Token::Comma {
            self.next_token(); 
            self.next_token(); 
            if let Token::Ident(name) = &self.cur_token {
                parameters.push(name.clone());
            }
            if self.peek_token == Token::Colon {
                self.next_token(); 
                self.next_token(); 
            }
        }
        
        if self.peek_token != Token::RParen {
            return None;
        }
        self.next_token(); 
        
        Some(parameters)
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        let is_mutable = self.cur_token == Token::Var;

        let name = match &self.peek_token {
            Token::Ident(name) => name.clone(),
            _ => return None,
        };
        self.next_token(); 

        if self.peek_token != Token::Assign {
            return None;
        }
        self.next_token(); 
        self.next_token(); 

        let value = self.parse_expression(Precedence::Lowest)?;

        Some(Statement::Let { name, value, is_mutable })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token(); 
        let value = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Return(value))
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let value = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Expression(value))
    }

    fn parse_block_statement(&mut self) -> Statement {
        let mut statements = Vec::new();
        self.next_token(); // Move past '{'

        while self.cur_token != Token::RBrace && self.cur_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Statement::Block(statements)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp = match &self.cur_token {
            Token::Ident(name) => Some(Expression::Identifier(name.clone())),
            Token::Int(val) => Some(Expression::IntegerLiteral(*val)),
            Token::Float(val) => Some(Expression::FloatLiteral(*val)),
            Token::String(_) => self.parse_string_literal(),
            Token::True => Some(Expression::Boolean(true)),
            Token::False => Some(Expression::Boolean(false)),
            Token::Null => Some(Expression::NullLiteral),
            Token::Minus | Token::Bang => self.parse_prefix_expression(),
            Token::LParen => self.parse_grouped_expression(),
            Token::If => self.parse_if_expression(),
            Token::While => self.parse_while_expression(),
            Token::For => self.parse_for_expression(),
            Token::Match => self.parse_match_expression(),
            Token::Func => self.parse_function_literal(),
            Token::LBracket => self.parse_array_literal(),
            Token::LBrace => self.parse_hash_literal(),
            _ => {
                self.errors.push(format!("No prefix parse function for {:?}", self.cur_token));
                None
            }
        }?;

        while self.peek_token != Token::Eof && precedence < self.peek_precedence() {
            match self.peek_token {
                Token::Plus | Token::Minus | Token::Asterisk | Token::Slash |
                Token::Equal | Token::NotEqual | Token::LessThan | Token::GreaterThan |
                Token::And | Token::Or => {
                    self.next_token(); 
                    left_exp = self.parse_infix_expression(left_exp)?;
                }
                Token::LParen => {
                    self.next_token();
                    left_exp = self.parse_call_expression(left_exp)?;
                }
                Token::LBracket => {
                    self.next_token();
                    left_exp = self.parse_index_expression(left_exp)?;
                }
                _ => return Some(left_exp),
            }
        }

        Some(left_exp)
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        self.next_token(); // Move past "if"
        
        let condition = self.parse_expression(Precedence::Lowest)?;
        
        if self.peek_token != Token::LBrace {
            self.errors.push("Expected { after if condition".to_string());
            return None;
        }
        self.next_token(); // Move to "{"
        
        let consequence = self.parse_block_statement();
        
        let mut alternative = None;
        if self.peek_token == Token::Else {
            self.next_token(); // Move to "else"
            
            if self.peek_token != Token::LBrace {
                self.errors.push("Expected { after else".to_string());
                return None;
            }
            self.next_token(); // Move to "{"
            
            alternative = Some(Box::new(self.parse_block_statement()));
        }

        Some(Expression::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative,
        })
    }

    fn parse_while_expression(&mut self) -> Option<Expression> {
        self.next_token(); // Move past 'while'

        let condition = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Token::LBrace {
            self.errors.push("Expected { after while condition".to_string());
            return None;
        }
        self.next_token(); // Move to '{'

        let body = self.parse_block_statement();

        Some(Expression::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn parse_for_expression(&mut self) -> Option<Expression> {
        self.next_token(); // Move past 'for'

        let variable = match &self.cur_token {
            Token::Ident(name) => name.clone(),
            _ => {
                self.errors.push(format!("Expected identifier after for, got {:?}", self.cur_token));
                return None;
            }
        };
        self.next_token(); 

        if self.cur_token != Token::In {
            self.errors.push(format!("Expected 'in' after for variable, got {:?}", self.cur_token));
            return None;
        }
        self.next_token(); 

        let iterable = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Token::LBrace {
            self.errors.push("Expected { after for iterable".to_string());
            return None;
        }
        self.next_token(); // Move to '{'

        let body = self.parse_block_statement();

        Some(Expression::For {
            variable,
            iterable: Box::new(iterable),
            body: Box::new(body),
        })
    }

    fn parse_match_expression(&mut self) -> Option<Expression> {
        self.next_token(); // move past 'match'

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Token::LBrace {
            self.errors.push(format!("Expected {{ after match value, got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // move to '{'

        let mut cases = Vec::new();

        while self.peek_token != Token::RBrace {
            self.next_token(); // move to pattern
            let pattern = self.parse_expression(Precedence::Lowest)?;

            if self.peek_token != Token::FatArrow {
                self.errors.push(format!("Expected => after match pattern, got {:?}", self.peek_token));
                return None;
            }
            self.next_token(); // move to '=>'
            self.next_token(); // move to body start (can be single expression or block or string etc)

            // The body could be an expression or a block, but our block parsing works for `{ ... }`.
            // Wait, if it's a single expression, we can wrap it in a block statement.
            let body = if self.cur_token == Token::LBrace {
                self.parse_block_statement()
            } else {
                let expr = self.parse_expression(Precedence::Lowest)?;
                Statement::Block(vec![Statement::Expression(expr)])
            };

            cases.push((pattern, Box::new(body)));

            if self.peek_token == Token::Comma {
                self.next_token(); // consume comma
            }
        }

        self.next_token(); // consume '}'

        Some(Expression::Match {
            value: Box::new(value),
            cases,
        })
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        self.next_token(); // Move past 'func'
        
        if self.cur_token != Token::LParen {
            self.errors.push(format!("Expected ( for function parameters, got {:?}", self.cur_token));
            return None;
        }
        
        let parameters = self.parse_function_parameters()?;
        
        if self.peek_token == Token::Arrow {
            self.next_token();
            self.next_token();
        }
        
        if self.peek_token != Token::LBrace {
            self.errors.push("Expected { for function body".to_string());
            return None;
        }
        self.next_token();
        
        let body = self.parse_block_statement();
        
        Some(Expression::FunctionLiteral {
            name: None,
            parameters,
            body: Box::new(body),
        })
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let operator = match self.cur_token {
            Token::Minus => "-",
            Token::Bang => "!",
            _ => return None,
        }.to_string();

        self.next_token(); 
        let right = self.parse_expression(Precedence::Prefix)?;
        
        Some(Expression::Prefix {
            operator,
            right: Box::new(right),
        })
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token(); // Move past '('

        let exp = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Token::RParen {
            self.errors.push(format!("Expected ) but got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // Move past ')'

        Some(exp)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let operator = match self.cur_token {
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Asterisk => "*",
            Token::Slash => "/",
            Token::Equal => "==",
            Token::NotEqual => "!=",
            Token::LessThan => "<",
            Token::GreaterThan => ">",
            Token::And => "&&",
            Token::Or => "||",
            _ => return None,
        }.to_string();

        let precedence = self.cur_precedence();
        self.next_token(); 
        
        let right = self.parse_expression(precedence)?;

        Some(Expression::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let arguments = self.parse_call_arguments()?;
        Some(Expression::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<Expression>> {
        let mut args = Vec::new();
        
        if self.peek_token == Token::RParen {
            self.next_token(); // Move to ')'
            return Some(args);
        }
        self.next_token(); // Move to first arg
        
        if let Some(arg) = self.parse_expression(Precedence::Lowest) {
            args.push(arg);
        }
        
        while self.peek_token == Token::Comma {
            self.next_token(); // move to comma
            self.next_token(); // move to next arg
            if let Some(arg) = self.parse_expression(Precedence::Lowest) {
                args.push(arg);
            }
        }
        
        if self.peek_token != Token::RParen {
            return None;
        }
        self.next_token(); // Move to ')'
        
        Some(args)
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let mut elements = Vec::new();

        if self.peek_token == Token::RBracket {
            self.next_token(); // Move to ']'
            return Some(Expression::Array(elements));
        }
        self.next_token(); // Move to first element

        if let Some(el) = self.parse_expression(Precedence::Lowest) {
            elements.push(el);
        }

        while self.peek_token == Token::Comma {
            self.next_token(); // move to comma
            self.next_token(); // move to next element
            if let Some(el) = self.parse_expression(Precedence::Lowest) {
                elements.push(el);
            }
        }

        if self.peek_token != Token::RBracket {
            self.errors.push(format!("Expected ] for array, got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // Move to ']'

        Some(Expression::Array(elements))
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let mut pairs = Vec::new();

        if self.peek_token == Token::RBrace {
            self.next_token(); // Move to '}'
            return Some(Expression::HashLiteral(pairs));
        }

        while self.peek_token != Token::RBrace {
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;

            if self.peek_token != Token::Colon {
                self.errors.push(format!("Expected : after hash key, got {:?}", self.peek_token));
                return None;
            }
            self.next_token(); // move to ':'
            self.next_token(); // move to value

            let value = self.parse_expression(Precedence::Lowest)?;
            pairs.push((key, value));

            if self.peek_token != Token::RBrace && self.peek_token != Token::Comma {
                self.errors.push(format!("Expected }} or , after hash pair, got {:?}", self.peek_token));
                return None;
            }

            if self.peek_token == Token::Comma {
                self.next_token(); // consume comma
            }
        }

        if self.peek_token != Token::RBrace {
            return None;
        }
        self.next_token(); // consume '}'

        Some(Expression::HashLiteral(pairs))
    }

    fn parse_index_expression(&mut self, left: Expression) -> Option<Expression> {
        self.next_token(); // move past [

        let index = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Token::RBracket {
            self.errors.push(format!("Expected ] for array index, got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // move past ]

        Some(Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    fn parse_string_literal(&mut self) -> Option<Expression> {
        let val = match &self.cur_token {
            Token::String(s) => s.clone(),
            _ => return None,
        };

        if !val.contains('{') {
            return Some(Expression::StringLiteral(val));
        }

        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = val.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if !current_text.is_empty() {
                    parts.push(Expression::StringLiteral(current_text.clone()));
                    current_text.clear();
                }
                
                let mut expr_str = String::new();
                let mut brace_count = 1;
                
                while let Some(inner_c) = chars.next() {
                    if inner_c == '{' {
                        brace_count += 1;
                    } else if inner_c == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                    expr_str.push(inner_c);
                }
                
                let inner_lexer = Lexer::new(&expr_str);
                let mut inner_parser = Parser::new(inner_lexer);
                if let Some(expr) = inner_parser.parse_expression(Precedence::Lowest) {
                    parts.push(expr);
                }
                self.errors.extend(inner_parser.errors);
            } else {
                current_text.push(c);
            }
        }
        
        if !current_text.is_empty() {
            parts.push(Expression::StringLiteral(current_text));
        }

        if parts.is_empty() {
            return Some(Expression::StringLiteral("".to_string()));
        }

        let mut combined = parts[0].clone();
        for i in 1..parts.len() {
            combined = Expression::Infix {
                left: Box::new(combined),
                operator: "+".to_string(),
                right: Box::new(parts[i].clone()),
            };
        }
        
        Some(combined)
    }
}
