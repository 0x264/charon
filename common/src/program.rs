use ahash::HashMap;
use crate::constant::ConstantItem;

pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Method>
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Method>) -> Self {
        Self { name, methods }
    }
}

pub struct Method {
    pub class_name: String,
    pub name: String,
    pub params: u8, // not include 'this'
    pub code: Vec<u8>
}

impl Method {
    pub fn new(class_name: String, name: String, params: u8, code: Vec<u8>) -> Self {
        Self { class_name, name, params, code }
    }
}

pub struct Function {
    pub name: String,
    pub params: u8,
    pub code: Vec<u8>
}

impl Function {
    pub fn new(name: String, params: u8, code: Vec<u8>) -> Self {
        Self { name, params, code }
    }
}

pub struct Program {
    pub minor: u8,
    pub major: u8,
    pub constant_pool: Vec<ConstantItem>,
    pub classes: HashMap<String, Class>,
    pub functions: HashMap<String, Function>
}

impl Program {
    pub fn new(minor: u8
               , major: u8
               , constant_pool: Vec<ConstantItem>
               , classes: HashMap<String, Class>
               , functions: HashMap<String, Function>
    ) -> Self {
        Self { minor, major, constant_pool, classes, functions }
    }
}