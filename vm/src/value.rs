use ahash::HashMap;

pub enum Value {
    Null,
    Long(i64),
    Double(f64),
    String(String),
    Class(String),
    Instance(Instance),
    Function(Function),
    Method(Method)
}

pub struct Instance {
    pub class_name: String,
    pub fields: HashMap<String, Value>
}

pub struct Function {
    pub name: String,
    pub params: u8
}

pub struct Method {
    pub class_name: String,
    pub name: String,
    pub params: u8
}