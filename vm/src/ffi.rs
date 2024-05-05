use std::collections::VecDeque;
use crate::value::Value;

pub trait Ffi {
    fn invoke(&self, args: VecDeque<Value>) -> Value;
}

pub struct StdPrint;

impl Ffi for StdPrint {
    fn invoke(&self, args: VecDeque<Value>) -> Value {
        if let Some(v) = args.front() {
            print(v);
        }
        
        Value::Null
    }
}

pub struct StdPrintln;

impl Ffi for StdPrintln {
    fn invoke(&self, args: VecDeque<Value>) -> Value {
        if let Some(v) = args.front() {
            print(v);
            println!();
        }

        Value::Null
    }
}

fn print(v: &Value) {
    match v {
        Value::Null => print!("null"),
        Value::True => print!("true"),
        Value::False => print!("false"),
        Value::Bool(v) => print!("{}", if *v {"true"} else {"false"}),
        Value::Long(v) => print!("{v}"),
        Value::Double(v) => print!("{v}"),
        Value::String(v) => print!("{v}"),
        Value::Class(c) => print!("<class: {}>", unsafe { &(**c).name }),
        Value::Instance(i) => print!("<class: {}'s instance>", unsafe {&(**i).class_name()}),
        Value::Function(f) => print!("<function: {}>", unsafe {&(**f).name}),
        Value::Method(m) => print!("<class: {}'s method: {}>", m.class_name(), m.name()),
        Value::ForeignFunction(ff) => print!("<foreign function: {}>", ff.name)
    }
}