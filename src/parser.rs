use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Logical,     // &&, ||
    Equals,      // ==, !=
    LessGreater, // <, >, <=, >=
    Range,       // .., ..=
    Sum,         // +, -
    Product,     // *, /, %
    Prefix,      // -X or !X
    Call,        // myFunction(X)
    Index,       // array[index]
    Dot,         // object.field
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Or | Token::And => Precedence::Logical,
        Token::Equal | Token::NotEqual => Precedence::Equals,
        Token::LessThan | Token::GreaterThan | Token::LessEqual | Token::GreaterEqual => Precedence::LessGreater,
        Token::DotDot | Token::DotDotEqual => Precedence::Range,
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Asterisk | Token::Slash | Token::Percent => Precedence::Product,
        Token::LParen => Precedence::Call,
        Token::LBracket => Precedence::Index,
        Token::Dot => Precedence::Dot,
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
    fn push_error(&mut self, msg: String) {
        let line_num = self.lexer.line;
        let col_num = self.lexer.column;
        
        let mut error_msg = format!("[Line {}, Col {}] {}", line_num, col_num, msg);
        
        if let Some(line_content) = self.lexer.get_line(line_num) {
            error_msg.push('\n');
            error_msg.push_str("    |\n");
            error_msg.push_str(&format!("{:3} | {}\n", line_num, line_content));
            error_msg.push_str("    | ");
            for _ in 0..col_num.saturating_sub(1) {
                error_msg.push(' ');
            }
            error_msg.push_str("^^^\n");
        }
        
        self.errors.push(error_msg);
    }

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
            Token::Struct => self.parse_struct_statement(),
            Token::Let | Token::Var => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => self.parse_break_statement(),
            Token::Continue => self.parse_continue_statement(),
            Token::Func => self.parse_func_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_struct_statement(&mut self) -> Option<Statement> {
        self.next_token(); // Move past 'struct'
        let name = match &self.cur_token {
            Token::Ident(name) => name.clone(),
            _ => {
                self.push_error("Expected struct name identifier".to_string());
                return None;
            }
        };

        if self.peek_token != Token::LBrace {
            self.push_error(format!("Expected {{ after struct name, got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // Move to '{'
        self.next_token(); // Move to first field or '}'

        let mut fields = Vec::new();
        while self.cur_token != Token::RBrace && self.cur_token != Token::Eof {
            let field_name = match &self.cur_token {
                Token::Ident(n) => n.clone(),
                _ => {
                    self.push_error(format!("Expected field name in struct, got {:?}", self.cur_token));
                    return None;
                }
            };

            let mut field_type = None;
            if self.peek_token == Token::Colon {
                self.next_token(); // Move to ':'
                self.next_token(); // Move to type ident
                if let Token::Ident(t) = &self.cur_token {
                    field_type = Some(t.clone());
                } else {
                    self.push_error(format!("Expected type annotation after ':', got {:?}", self.cur_token));
                    return None;
                }
            }

            fields.push((field_name, field_type));

            if self.peek_token == Token::Comma {
                self.next_token(); // consume ','
            }
            self.next_token(); // move to next field or '}'
        }

        if self.cur_token != Token::RBrace {
            self.push_error(format!("Expected }} to close struct definition, got {:?}", self.cur_token));
            return None;
        }

        Some(Statement::StructDef { name, fields })
    }

    fn parse_func_statement(&mut self) -> Option<Statement> {
        self.next_token();
        
        let name = match &self.cur_token {
            Token::Ident(name) => name.clone(),
            _ => {
                self.push_error("Expected function name".to_string());
                return None;
            }
        };
        self.next_token(); 
        
        let parameters = self.parse_function_parameters()?;
        
        let mut return_type = None;
        if self.peek_token == Token::Arrow {
            self.next_token();
            self.next_token();
            if let Token::Ident(type_name) = &self.cur_token {
                return_type = Some(type_name.clone());
            }
        }
        
        if self.peek_token != Token::LBrace {
            self.push_error("Expected { after function parameters".to_string());
            return None;
        }
        self.next_token(); 
        
        let body = self.parse_block_statement();
        
        let function_literal = Expression::FunctionLiteral {
            name: Some(name.clone()),
            parameters,
            return_type,
            body: Box::new(body),
        };

        Some(Statement::Let {
            name,
            value: function_literal,
            is_mutable: false, // Functions are immutable by default
        })
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<(String, Option<String>)>> {
        let mut parameters = Vec::new();
        
        if self.peek_token == Token::RParen {
            self.next_token(); 
            return Some(parameters);
        }
        self.next_token(); 
        
        if let Token::Ident(name_ref) = &self.cur_token {
            let name = name_ref.clone();
            let mut param_type = None;
            if self.peek_token == Token::Colon {
                self.next_token(); // consume ':'
                self.next_token(); // consume type ident
                if let Token::Ident(type_name) = &self.cur_token {
                    param_type = Some(type_name.clone());
                }
            }
            parameters.push((name, param_type));
        }

        while self.peek_token == Token::Comma {
            self.next_token(); // consume ','
            self.next_token(); // move to next ident
            if let Token::Ident(name_ref) = &self.cur_token {
                let name = name_ref.clone();
                let mut param_type = None;
                if self.peek_token == Token::Colon {
                    self.next_token(); // consume ':'
                    self.next_token(); // move to type ident
                    if let Token::Ident(type_name) = &self.cur_token {
                        param_type = Some(type_name.clone());
                    }
                }
                parameters.push((name, param_type));
            }
        }
        
        if self.peek_token != Token::RParen {
            self.push_error(format!("Expected ) for parameters, got {:?}", self.peek_token));
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

    fn parse_break_statement(&mut self) -> Option<Statement> {
        Some(Statement::Break)
    }

    fn parse_continue_statement(&mut self) -> Option<Statement> {
        Some(Statement::Continue)
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(Precedence::Lowest)?;
        
        if matches!(self.peek_token, Token::Assign | Token::PlusAssign | Token::MinusAssign | Token::AsteriskAssign | Token::SlashAssign | Token::PercentAssign) {
            let op_token = self.peek_token.clone();
            self.next_token(); // Move to '=' or '+=' etc.
            self.next_token(); // Move to expression
            
            let value_expr = self.parse_expression(Precedence::Lowest)?;
            let final_value = match op_token {
                Token::Assign => value_expr,
                Token::PlusAssign => Expression::Infix {
                    left: Box::new(expr.clone()),
                    operator: "+".to_string(),
                    right: Box::new(value_expr),
                },
                Token::MinusAssign => Expression::Infix {
                    left: Box::new(expr.clone()),
                    operator: "-".to_string(),
                    right: Box::new(value_expr),
                },
                Token::AsteriskAssign => Expression::Infix {
                    left: Box::new(expr.clone()),
                    operator: "*".to_string(),
                    right: Box::new(value_expr),
                },
                Token::SlashAssign => Expression::Infix {
                    left: Box::new(expr.clone()),
                    operator: "/".to_string(),
                    right: Box::new(value_expr),
                },
                Token::PercentAssign => Expression::Infix {
                    left: Box::new(expr.clone()),
                    operator: "%".to_string(),
                    right: Box::new(value_expr),
                },
                _ => return None,
            };

            return match expr {
                Expression::Identifier(name) => Some(Statement::Assign { name, value: final_value }),
                Expression::Index { left, index } => Some(Statement::IndexAssign { left: *left, index: *index, value: final_value }),
                Expression::FieldAccess { object, field } => Some(Statement::FieldAssign { object: *object, field, value: final_value }),
                _ => {
                    self.push_error(format!("invalid assignment target: {:?}", expr));
                    None
                }
            };
        }

        Some(Statement::Expression(expr))
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
            Token::Try => self.parse_try_expression(),
            Token::Throw => self.parse_throw_expression(),
            Token::Func => self.parse_function_literal(),
            Token::LBracket => self.parse_array_literal(),
            Token::LBrace => self.parse_hash_literal(),
            _ => {
                self.push_error(format!("No prefix parse function for {:?}", self.cur_token));
                None
            }
        }?;

        while self.peek_token != Token::Eof && precedence < self.peek_precedence() {
            match self.peek_token {
                Token::Plus | Token::Minus | Token::Asterisk | Token::Slash | Token::Percent |
                Token::Equal | Token::NotEqual | Token::LessThan | Token::GreaterThan |
                Token::LessEqual | Token::GreaterEqual |
                Token::And | Token::Or => {
                    self.next_token(); 
                    left_exp = self.parse_infix_expression(left_exp)?;
                }
                Token::DotDot | Token::DotDotEqual => {
                    let inclusive = self.peek_token == Token::DotDotEqual;
                    self.next_token();
                    let prec = self.cur_precedence();
                    self.next_token();
                    let right = self.parse_expression(prec)?;
                    left_exp = Expression::Range {
                        start: Box::new(left_exp),
                        end: Box::new(right),
                        inclusive,
                    };
                }
                Token::LParen => {
                    self.next_token();
                    left_exp = self.parse_call_expression(left_exp)?;
                }
                Token::LBracket => {
                    self.next_token();
                    left_exp = self.parse_index_expression(left_exp)?;
                }
                Token::Dot => {
                    self.next_token(); // Move to '.'
                    if let Token::Ident(field_name) = &self.peek_token {
                        let field = field_name.clone();
                        self.next_token(); // Move to identifier
                        left_exp = Expression::FieldAccess {
                            object: Box::new(left_exp),
                            field,
                        };
                    } else {
                        self.push_error(format!("Expected identifier after '.', got {:?}", self.peek_token));
                        return None;
                    }
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
            self.push_error("Expected { after if condition".to_string());
            return None;
        }
        self.next_token(); // Move to "{"
        
        let consequence = self.parse_block_statement();
        
        let mut alternative = None;
        if self.peek_token == Token::Else {
            self.next_token(); // Move to "else"
            
            if self.peek_token != Token::LBrace {
                self.push_error("Expected { after else".to_string());
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
            self.push_error("Expected { after while condition".to_string());
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
                self.push_error(format!("Expected identifier after for, got {:?}", self.cur_token));
                return None;
            }
        };
        self.next_token(); 

        if self.cur_token != Token::In {
            self.push_error(format!("Expected 'in' after for variable, got {:?}", self.cur_token));
            return None;
        }
        self.next_token(); 

        let iterable = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Token::LBrace {
            self.push_error("Expected { after for iterable".to_string());
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
            self.push_error(format!("Expected {{ after match value, got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // move to '{'

        let mut cases = Vec::new();

        while self.peek_token != Token::RBrace {
            self.next_token(); // move to pattern
            let pattern = self.parse_expression(Precedence::Lowest)?;

            if self.peek_token != Token::FatArrow {
                self.push_error(format!("Expected => after match pattern, got {:?}", self.peek_token));
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

    fn parse_throw_expression(&mut self) -> Option<Expression> {
        self.next_token(); // Move past 'throw'
        let exp = self.parse_expression(Precedence::Lowest)?;
        Some(Expression::Throw(Box::new(exp)))
    }

    fn parse_try_expression(&mut self) -> Option<Expression> {
        self.next_token(); // Move past 'try'

        if self.cur_token != Token::LBrace {
            self.push_error(format!("Expected {{ after try, got {:?}", self.cur_token));
            return None;
        }

        let try_body = self.parse_block_statement();

        if self.peek_token != Token::Catch {
            self.push_error(format!("Expected catch after try block, got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // Move to 'catch'
        self.next_token(); // Move to identifier

        let catch_param = match &self.cur_token {
            Token::Ident(name) => name.clone(),
            _ => {
                self.push_error(format!("Expected identifier after catch, got {:?}", self.cur_token));
                return None;
            }
        };

        if self.peek_token != Token::LBrace {
            self.push_error(format!("Expected {{ after catch parameter, got {:?}", self.peek_token));
            return None;
        }
        self.next_token(); // move to '{'

        let catch_body = self.parse_block_statement();

        Some(Expression::TryCatch {
            try_body: Box::new(try_body),
            catch_param,
            catch_body: Box::new(catch_body),
        })
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        self.next_token(); // Move past 'func'
        
        if self.cur_token != Token::LParen {
            self.push_error(format!("Expected ( for function parameters, got {:?}", self.cur_token));
            return None;
        }
        
        let parameters = self.parse_function_parameters()?;
        
        let mut return_type = None;
        if self.peek_token == Token::Arrow {
            self.next_token(); // consume ')'
            self.next_token(); // move to type ident
            if let Token::Ident(type_name) = &self.cur_token {
                return_type = Some(type_name.clone());
            }
        }
        
        if self.peek_token != Token::LBrace {
            self.push_error("Expected { for function body".to_string());
            return None;
        }
        self.next_token();
        
        let body = self.parse_block_statement();
        
        Some(Expression::FunctionLiteral {
            name: None,
            parameters,
            return_type,
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
            self.push_error(format!("Expected ) but got {:?}", self.peek_token));
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
            Token::Percent => "%",
            Token::Equal => "==",
            Token::NotEqual => "!=",
            Token::LessThan => "<",
            Token::GreaterThan => ">",
            Token::LessEqual => "<=",
            Token::GreaterEqual => ">=",
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
            self.push_error(format!("Expected ] for array, got {:?}", self.peek_token));
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
                self.push_error(format!("Expected : after hash key, got {:?}", self.peek_token));
                return None;
            }
            self.next_token(); // move to ':'
            self.next_token(); // move to value

            let value = self.parse_expression(Precedence::Lowest)?;
            pairs.push((key, value));

            if self.peek_token != Token::RBrace && self.peek_token != Token::Comma {
                self.push_error(format!("Expected }} or , after hash pair, got {:?}", self.peek_token));
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
            self.push_error(format!("Expected ] for array index, got {:?}", self.peek_token));
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

        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = val.chars().peekable();
        let mut has_interpolation = false;

        while let Some(c) = chars.next() {
            if c == '\\' && chars.peek() == Some(&'{') {
                chars.next();
                current_text.push('{');
            } else if c == '\\' && chars.peek() == Some(&'}') {
                chars.next();
                current_text.push('}');
            } else if c == '{' {
                has_interpolation = true;
                if !current_text.is_empty() {
                    parts.push(Expression::StringLiteral(current_text.clone()));
                    current_text.clear();
                }
                
                let mut expr_str = String::new();
                let mut brace_count = 1;
                let mut in_string = false;
                
                while let Some(inner_c) = chars.next() {
                    if in_string {
                        if inner_c == '\\' {
                            expr_str.push(inner_c);
                            if let Some(next_c) = chars.next() {
                                expr_str.push(next_c);
                            }
                            continue;
                        } else if inner_c == '"' {
                            in_string = false;
                        }
                    } else {
                        if inner_c == '"' {
                            in_string = true;
                        } else if inner_c == '{' {
                            brace_count += 1;
                        } else if inner_c == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                break;
                            }
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
        
        if !has_interpolation {
            return Some(Expression::StringLiteral(current_text));
        }

        if !current_text.is_empty() {
            parts.push(Expression::StringLiteral(current_text));
        }

        if parts.is_empty() {
            return Some(Expression::StringLiteral("".to_string()));
        }

        let mut combined = match &parts[0] {
            Expression::StringLiteral(_) => parts[0].clone(),
            _ => Expression::StringLiteral("".to_string()),
        };
        let start_idx = match &parts[0] {
            Expression::StringLiteral(_) => 1,
            _ => 0,
        };
        for i in start_idx..parts.len() {
            combined = Expression::Infix {
                left: Box::new(combined),
                operator: "+".to_string(),
                right: Box::new(parts[i].clone()),
            };
        }
        
        Some(combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_let_statements() {
        let input = "
            let x = 5
            let y = 10
            let foobar = 838383
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_compound_assignments() {
        let input = "
            x += 5
            y -= 3
            z *= 2
            a /= 4
            b %= 7
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 5);

        assert_eq!(
            program.statements[0],
            Statement::Assign {
                name: "x".to_string(),
                value: Expression::Infix {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    operator: "+".to_string(),
                    right: Box::new(Expression::IntegerLiteral(5)),
                }
            }
        );
        assert_eq!(
            program.statements[4],
            Statement::Assign {
                name: "b".to_string(),
                value: Expression::Infix {
                    left: Box::new(Expression::Identifier("b".to_string())),
                    operator: "%".to_string(),
                    right: Box::new(Expression::IntegerLiteral(7)),
                }
            }
        );
    }

    #[test]
    fn test_range_expressions() {
        let input = "
            0..10
            1..=5
            0 + 1 .. 9 + 1
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 3);

        assert_eq!(
            program.statements[0],
            Statement::Expression(Expression::Range {
                start: Box::new(Expression::IntegerLiteral(0)),
                end: Box::new(Expression::IntegerLiteral(10)),
                inclusive: false,
            })
        );
        assert_eq!(
            program.statements[1],
            Statement::Expression(Expression::Range {
                start: Box::new(Expression::IntegerLiteral(1)),
                end: Box::new(Expression::IntegerLiteral(5)),
                inclusive: true,
            })
        );
    }

    #[test]
    fn test_escaped_braces_in_string() {
        let input = r#"
            let s = "Literal \{name\} here"
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(
            program.statements[0],
            Statement::Let {
                name: "s".to_string(),
                value: Expression::StringLiteral("Literal {name} here".to_string()),
                is_mutable: false,
            }
        );
    }
}

#[cfg(test)]
mod tests_control_flow {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_break_continue_statements() {
        let input = "
            break
            continue
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(parser.errors.len(), 0);
        assert_eq!(program.statements.len(), 2);
        assert_eq!(program.statements[0], Statement::Break);
        assert_eq!(program.statements[1], Statement::Continue);
    }

    #[test]
    fn test_index_and_field_assignments() {
        let input = "
            arr[0] = 5
            matrix[1][2] = 99
            arr[i] += 1
            p.x = 10
            p.x += 5
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 5);

        assert_eq!(
            program.statements[0],
            Statement::IndexAssign {
                left: Expression::Identifier("arr".to_string()),
                index: Expression::IntegerLiteral(0),
                value: Expression::IntegerLiteral(5),
            }
        );
        assert_eq!(
            program.statements[1],
            Statement::IndexAssign {
                left: Expression::Index {
                    left: Box::new(Expression::Identifier("matrix".to_string())),
                    index: Box::new(Expression::IntegerLiteral(1)),
                },
                index: Expression::IntegerLiteral(2),
                value: Expression::IntegerLiteral(99),
            }
        );
        assert_eq!(
            program.statements[3],
            Statement::FieldAssign {
                object: Expression::Identifier("p".to_string()),
                field: "x".to_string(),
                value: Expression::IntegerLiteral(10),
            }
        );
    }

    #[test]
    fn test_struct_definition() {
        let input = "
            struct Point {
                x: Int,
                y: Int
            }
            struct User {
                id: Int,
                name: String,
                is_admin: Bool
            }
        ";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 2);

        assert_eq!(
            program.statements[0],
            Statement::StructDef {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Some("Int".to_string())),
                    ("y".to_string(), Some("Int".to_string())),
                ],
            }
        );
    }
}
