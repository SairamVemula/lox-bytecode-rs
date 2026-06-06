use crate::{
    chunk::{Chunk, OpCode},
    scanner::Scanner,
    token::{Token, TokenType},
    value::Value,
    vm::InterpretError,
};

#[derive(Debug)]
pub struct Compiler {
    parser: Parser,
    scanner: Scanner,
    chunk: Chunk,
}

#[derive(Debug)]
pub struct Parser {
    previous: Option<Token>,
    current: Option<Token>,
    panic_mode: bool,
    had_error: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl From<usize> for Precedence {
    fn from(value: usize) -> Self {
        use Precedence::*;
        match value {
            1 => Assignment,
            2 => Or,
            3 => And,
            4 => Equality,
            5 => Comparison,
            6 => Term,
            7 => Factor,
            8 => Unary,
            9 => Call,
            10 => Primary,
            o => panic!("Cannot convert {o} into Precedence"),
        }
    }
}

impl Precedence {
    fn next(self) -> Self {
        if self == Precedence::Primary {
            panic!("No next after Primary")
        }
        ((self as usize) + 1).into()
    }
    fn previous(self) -> Self {
        if self == Precedence::None {
            panic!("No next before Primary")
        }
        ((self as usize) - 1).into()
    }
}

type ParseRuleFn = fn(&mut Compiler);
#[derive(Clone, Copy)]
pub struct ParseRule {
    prefix: Option<ParseRuleFn>,
    infix: Option<ParseRuleFn>,
    precedence: Precedence,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            parser: Parser {
                previous: None,
                current: None,
                panic_mode: false,
                had_error: false,
            },
            scanner: Scanner::new(""),
            chunk: Chunk::new(),
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<Chunk, InterpretError> {
        self.scanner = Scanner::new(source);
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();
        if self.parser.had_error {
            return Err(InterpretError::CompileError);
        }
        Ok(std::mem::replace(&mut self.chunk, Chunk::new()))
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.as_ref().unwrap().token_type;
        let rule = Self::get_rule(&operator_type);
        self.parse_precedence(rule.precedence.next());
        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiple.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal.into(), OpCode::Not.into()),
            TokenType::Equals => self.emit_byte(OpCode::Equal.into()),
            TokenType::Greater => self.emit_byte(OpCode::Greater.into()),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less.into(), OpCode::Not.into()),
            TokenType::Less => self.emit_byte(OpCode::Less.into()),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater.into(), OpCode::Not.into()),
            _ => todo!(),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.as_ref().unwrap().token_type;
        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            TokenType::Bang => self.emit_byte(OpCode::Not.into()),
            _ => (),
        }
    }

    fn number(&mut self) {
        let lexeme = self.parser.previous.as_ref().unwrap().lexeme.clone();
        let value: f64 = lexeme.parse().unwrap();
        match self.chunk.add_constant(Value::Number(value)) {
            Ok(constant) => self.emit_bytes(OpCode::Constant.into(), constant),
            Err(_) => self.error_at_previous("Too many constants in one chunk."),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.as_ref().unwrap().token_type {
            TokenType::True => self.emit_byte(OpCode::True.into()),
            TokenType::False => self.emit_byte(OpCode::False.into()),
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            _ => unimplemented!("literal type not implemented"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = Self::get_rule(&self.parser.previous.as_ref().unwrap().token_type).prefix;
        match prefix_rule {
            Some(prefix) => prefix(self),
            None => {
                self.error_at_current("Expect expression.");
                return;
            }
        }

        while precedence
            <= Self::get_rule(&self.parser.current.as_ref().unwrap().token_type).precedence
        {
            self.advance();
            let infix_rule =
                Self::get_rule(&self.parser.previous.as_ref().unwrap().token_type).infix;
            match infix_rule {
                Some(infix) => infix(self),
                None => return,
            }
        }
    }

    fn get_rule(token_type: &TokenType) -> ParseRule {
        use TokenType::*;
        match token_type {
            LeftParen => ParseRule {
                prefix: Some(Compiler::grouping),
                infix: None,
                precedence: Precedence::None,
            },
            RightParen => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            LeftBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            RightBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Dot => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Minus => ParseRule {
                prefix: Some(Compiler::unary),
                infix: Some(Compiler::binary),
                precedence: Precedence::Term,
            },
            Plus => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Term,
            },
            Semicolon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Slash => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Factor,
            },
            Star => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Factor,
            },
            Bang => ParseRule {
                prefix: Some(Compiler::unary),
                infix: None,
                precedence: Precedence::None,
            },
            BangEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Equality,
            },
            Assign => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Equals => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Equality,
            },
            Greater => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            Less => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            LessEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            Identifier => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            String => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Number => ParseRule {
                prefix: Some(Compiler::number),
                infix: None,
                precedence: Precedence::None,
            },
            And => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Class => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            False => ParseRule {
                prefix: Some(Compiler::literal),
                infix: None,
                precedence: Precedence::None,
            },
            For => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Fun => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            If => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Nil => ParseRule {
                prefix: Some(Compiler::literal),
                infix: None,
                precedence: Precedence::None,
            },
            Or => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Print => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Return => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Super => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            This => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            True => ParseRule {
                prefix: Some(Compiler::literal),
                infix: None,
                precedence: Precedence::None,
            },
            Var => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }

    fn error_at_previous(&mut self, message: &str) {
        if let Some(token) = self.parser.previous.take() {
            self.error_at(&token, message);
            self.parser.previous = Some(token);
        }
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.take();

        loop {
            let token = self.scanner.scan_token();

            if token.token_type == TokenType::Error {
                self.error_at(&token, &token.lexeme);
                continue;
            }

            self.parser.current = Some(token);
            break;
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.parser.current.as_ref().unwrap().token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = match self.parser.previous.as_ref() {
            Some(p) => p.line,
            None => 1,
        };
        self.chunk.write(byte, line);
    }

    fn emit_bytes(&mut self, a: u8, b: u8) {
        self.emit_byte(a);
        self.emit_byte(b);
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        #[cfg(feature = "debug_trace_execution")]
        if self.parser.had_error {
            self.chunk.disassemble("code");
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.into());
    }

    fn error_at_current(&mut self, message: &str) {
        if let Some(token) = self.parser.current.take() {
            self.error_at(&token, message);
            self.parser.current = Some(token);
        }
    }

    fn error(&mut self, token: &Token, message: &str) {
        self.error_at(token, message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.parser.panic_mode {
            return;
        }

        self.parser.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        if token.token_type == TokenType::Eof {
            eprint!(" at end");
        } else if token.token_type != TokenType::Error {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {message}");

        self.parser.had_error = true;
    }
}
