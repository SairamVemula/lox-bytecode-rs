use crate::{
    chunk::{Chunk, OpCode},
    compiler::Compiler,
    value::Value,
};

pub type InterpretResult<T = ()> = Result<T, InterpretError>;

pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut compiler = Compiler::new();
        let chunk = compiler.compile(source)?;
        self.ip = 0;
        self.run(&chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {slot} ]");
                }
                println!();
                chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte(chunk);

            match instruction {
                OpCode::Return => {
                    println!("{}", self.pop());
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant(chunk);
                    self.stack.push(constant);
                }
                OpCode::Negate => {
                    if let Value::Number(_) = self.peek(0) {
                        let value = self.pop();
                        self.stack.push(-value);
                    } else {
                        return self.runtime_error(chunk, "Operand must be numbers.");
                    }
                }
                OpCode::Add => self.binary_op(chunk, |a, b| a + b)?,
                OpCode::Subtract => self.binary_op(chunk, |a, b| a - b)?,
                OpCode::Multiple => self.binary_op(chunk, |a, b| a * b)?,
                OpCode::Divide => self.binary_op(chunk, |a, b| a / b)?,
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Boolean(true)),
                OpCode::False => self.stack.push(Value::Boolean(false)),
                OpCode::Not => {
                    let value = self.pop();
                    self.stack.push(Value::Boolean(value.is_falsy()))
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Greater => self.binary_op(chunk, |a, b| Value::Boolean(a > b))?,
                OpCode::Less => self.binary_op(chunk, |a, b| Value::Boolean(a < b))?,
            }
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack.len() - distance - 1]
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn read_byte(&mut self, chunk: &Chunk) -> OpCode {
        let val = chunk.read(self.ip).into();
        self.ip += 1;
        val
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        let index = chunk.read(self.ip).into();
        self.ip += 1;
        chunk.get_constant(index)
    }

    fn binary_op(&mut self, chunk: &Chunk, op: fn(a: Value, b: Value) -> Value) -> InterpretResult {
        if !self.peek(0).is_number() || !self.peek(1).is_number() {
            return self.runtime_error(chunk, "Operand must be numbers.");
        }
        let b = self.pop();
        let a = self.pop();
        self.stack.push(op(a, b));
        Ok(())
    }

    fn runtime_error(&mut self, chunk: &Chunk, msg: &str) -> InterpretResult {
        let line = chunk.get_line(self.ip - 1);
        eprintln!("{msg}");
        eprintln!("[line {line}] in script");
        self.reset_stack();
        Err(InterpretError::RuntimeError)
    }
}
