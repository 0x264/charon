use std::cell::RefCell;
use std::rc::Rc;
use ahash::HashMap;
use common::program::{Class, Function, Method};
use crate::ffi::Ffi;

#[derive(PartialEq, Clone)]
pub enum Value {
    Null,
    True,
    False,
    Bool(bool),
    Long(i64),
    Double(f64),
    String(String),
    Class(*const Class),
    Instance(Rc<RefCell<Instance>>),
    Function(*const Function),
    Method(*const Method),
    ForeignFunction(ForeignFunction)
}

impl Value {
    pub unsafe fn as_bool_unchecked(&self) -> bool {
        match self {
            Value::Bool(v) => *v,
            _ => unreachable!()
        }
    }
    pub unsafe fn as_long_unchecked(&self) -> i64 {
        match self { 
            Value::Long(v) => *v,
            _ => unreachable!()
        }
    }
    pub unsafe fn as_double_unchecked(&self) -> f64 {
        match self { 
            Value::Double(v) => *v,
            _ => unreachable!()
        }
    }
    pub unsafe fn as_string_unchecked(&self) -> &str {
        match self { 
            Value::String(v) => v,
            _ => unreachable!()
        }
    }
    pub unsafe fn as_instance_unchecked(&self) -> &Rc<RefCell<Instance>> {
        match self { 
            Value::Instance(v) => v,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq)]
pub struct Instance {
    pub class: *const Class,
    pub fields: HashMap<String, Value>
}

#[derive(Clone)]
pub struct ForeignFunction {
    pub name: String,
    pub params: u8,
    pub entry: Rc<dyn Ffi>
}

impl PartialEq for ForeignFunction {
    fn eq(&self, other: &Self) -> bool {
        if self.params != other.params
            || self.name != other.name {
            return false;
        }
        std::ptr::addr_eq(Rc::as_ptr(&self.entry), Rc::as_ptr(&other.entry))
    }
}