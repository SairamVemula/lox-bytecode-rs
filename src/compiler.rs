use crate::{scanner::Scanner, token::TokenType};

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, source: &String) {
        let mut scanner = Scanner::new(source);
        let mut line = 0;
        let tokens = scanner.parse();
        println!("tokens = {}", tokens.len());
        for token in tokens {
            if token.line != line {
                print!("{:4} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!("{:10?} '{}'", token.token_type, token.lexeme);

            if token.token_type == TokenType::Eof {
                break;
            }
        }
    }
}
