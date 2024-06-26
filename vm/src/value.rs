use std::rc::Rc;
use ahash::{HashMap, HashMapExt};
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
    Instance(*mut Instance),
    Function(*const Function),
    Method(MemMethod),
    ForeignFunction(ForeignFunction)
}

impl Value {
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
}

#[derive(PartialEq)]
pub struct Instance {
    pub class: *const Class,
    pub fields: HashMap<String, Value>
}

impl Instance {
    pub fn new(class: *const Class) -> Self {
        Self {
            class,
            fields: HashMap::new()
        }
    }
    
    pub fn class_name(&self) -> &str {
        unsafe {&(*self.class).name}
    }
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

#[derive(Clone, PartialEq)]
pub struct MemMethod {
    pub instance: *mut Instance,
    pub method: *const Method
}

impl MemMethod {
    pub fn new(instance: *mut Instance, method: *const Method) -> Self {
        Self { instance, method }
    }
    
    pub fn param_count(&self) -> u8 {
        unsafe {(*self.method).params}
    }
    
    pub fn max_locals(&self) -> u8 {
        unsafe {(*self.method).max_locals}
    }
    
    pub fn name(&self) -> &str {
        unsafe {&(*self.method).name}
    }

    pub fn class_name(&self) -> &str {
        unsafe {&(*self.method).class_name}
    }
}
