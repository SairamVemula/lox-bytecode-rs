pub type Value = f64;

pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) -> u8 {
        let len = self.values.len();
        self.values.push(value);
        len as u8
    }

    pub fn free(&mut self) {
        self.values = Vec::new();
    }

    pub fn print_value(&self, index: usize) {
        print!("{}", self.values[index])
    }

    pub fn get_value(&self, index: usize) -> Value {
        self.values[index]
    }
}
