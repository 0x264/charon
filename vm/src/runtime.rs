use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::mem;
use std::process::exit;
use std::rc::Rc;
use ahash::{HashMap, HashMapExt};
use common::constant::{ConstantItem, ENTRY_NAME};
use common::program::{Class, Function, Method, Program};
use common::reader::LEReader;
use common::Result;
use common::opcode::*;
use crate::ffi::StdPrint;
use crate::stack::Stack;
use crate::value::{ForeignFunction, Instance, MemMethod, Value};

enum FrameType {
    Func(*const Function),
    Method(*const Method)
}

struct Frame {
    frame_type: FrameType,
    pc: Cell<usize>,
    sb: Cell<usize>,
    sp: Cell<usize>
}

impl Frame {
    fn new(frame_type: FrameType) -> Self {
        Self {
            frame_type,
            pc: Cell::new(0),
            sb: Cell::new(0),
            sp: Cell::new(0)
        }
    }

    fn code(&self) -> &[u8] {
        match self.frame_type {
            FrameType::Func(f) => unsafe {&(*f).code},
            FrameType::Method(m) => unsafe {&(*m).code}
        }
    }
}


macro_rules! bin_op {
    ($frame:ident, $stack:ident, $op:tt, $s:literal) => {{
        let r = pop_stack($frame, $stack);
        let l = pop_stack($frame, $stack);

        let res = match l {
            Value::Long(l) => match r {
                Value::Long(r) => Value::Long(l $op r),
                Value::Double(r) => Value::Double(l as f64 $op r),
                _ => return Err(format!("`{}`'s right operand can only support long & double when left operand is long", $s))
            }
            Value::Double(l) => match r {
                Value::Long(r) => Value::Double(l $op r as f64),
                Value::Double(r) => Value::Double(l $op r),
                _ => return Err(format!("`{}`'s right operand can only support long & double when left operand is double", $s))
            }
            _ => return Err(format!("`{}` can only used between long and double", $s))
        };
        push_stack($frame, $stack, res);
    }};
}

macro_rules! cmp_op {
    ($frame:ident, $stack:ident, $op:tt) => {{
        let r = pop_stack($frame, $stack);
        let l = pop_stack($frame, $stack);
        let res = if mem::discriminant(&l) != mem::discriminant(&r) {
            false
        } else {
            match l {
                Value::Long(l) => l $op unsafe {r.as_long_unchecked()},
                Value::Double(l) => l $op unsafe {r.as_double_unchecked()},
                Value::String(l) => l.as_str() $op unsafe {r.as_string_unchecked()},
                _ => false
            }
        };
        push_stack($frame, $stack, Value::Bool(res));
    }};
}

pub fn exec(program: Program) -> Result<()> {
    let Some(entry) = program.functions.get(ENTRY_NAME) else {
        return Err("failed to get program's entry".to_owned());
    };

    let mut frames = Vec::<Frame>::new();
    let stack = Stack::<Value>::new()?;

    let mut globals = HashMap::<String, Value>::new();

    // define classes & functions as globals
    for class in program.classes.values() {
        globals.insert(class.name.clone(), Value::Class(class as *const Class));
    }
    for func in program.functions.values() {
        globals.insert(func.name.clone(), Value::Function(func as *const Function));
    }
    
    // std function
    {
        let ff = ForeignFunction {
            name: "__print".to_owned(),
            params: 1,
            entry: Rc::new(StdPrint)
        };
        globals.insert(ff.name.clone(), Value::ForeignFunction(ff));
    }

    // create first frame
    frames.push(Frame::new(FrameType::Func(entry as *const Function)));

    loop {
        let Some(frame) = frames.last() else {
            break;
        };

        match run_code(frame, &stack, &mut globals, &program) {
            Ok(res) => match res {
                Some(new_frame) => frames.push(new_frame),
                None => {
                    let return_value = pop_stack(frame, &stack);
                    frames.pop();
                    if let Some(frame) = frames.last() {
                        push_stack(frame, &stack, return_value);
                    }
                }
            }
            Err(e) => print_error_and_exit(&e, &frames)
        }
    }

    Ok(())
}

fn run_code(frame: &Frame, stack: &Stack<Value>, globals: &mut HashMap<String, Value>, program: &Program) -> Result<Option<Frame>> {
    let mut reader = LEReader::new(frame.code());
    reader.set_offset(frame.pc.get())?;

    while let Ok(opcode) = reader.next_u8() {
        match opcode {
            OP_CONST_NULL => push_stack(frame, stack, Value::Null),
            OP_CONST_TRUE => push_stack(frame, stack, Value::True),
            OP_CONST_FALSE => push_stack(frame, stack, Value::False),

            OP_LCONST_M1 => push_stack(frame, stack, Value::Long(-1)),
            OP_LCONST_0 => push_stack(frame, stack, Value::Long(0)),
            OP_LCONST_1 => push_stack(frame, stack, Value::Long(1)),
            OP_LCONST_2 => push_stack(frame, stack, Value::Long(2)),
            OP_LCONST_3 => push_stack(frame, stack, Value::Long(3)),
            OP_LCONST_4 => push_stack(frame, stack, Value::Long(4)),
            OP_LCONST_5 => push_stack(frame, stack, Value::Long(5)),

            OP_LDC => {
                let idx = reader.next_u16()?;
                let Some(item) = program.constant_pool.get(idx as usize) else {
                    return Err(format!("`op_ldc`'s argument: {idx} not found in constant pool"));
                };
                match item {
                    ConstantItem::Long(v) => push_stack(frame, stack, Value::Long(*v)),
                    ConstantItem::Double(v) => push_stack(frame, stack, Value::Double(*v)),
                    ConstantItem::String(v) => push_stack(frame, stack, Value::String(v.to_owned()))
                }
            }

            OP_NEG => {
                let v = pop_stack(frame, stack);
                let res = match v {
                    Value::Long(v) => Value::Long(-v),
                    Value::Double(v) => Value::Double(-v),
                    _ => return Err("`-` can only apply to long & double".to_owned())
                };
                push_stack(frame, stack, res);
            }

            OP_ADD => {
                let r = pop_stack(frame, stack);
                let l = pop_stack(frame, stack);

                let res = match l {
                    Value::Long(l) => match r {
                        Value::Long(r) => Value::Long(l + r),
                        Value::Double(r) => Value::Double(l as f64 + r),
                        _ => return Err("`+`'s right operand can only support long & double when left operand is long".to_owned())
                    }
                    Value::Double(l) => match r {
                        Value::Long(r) => Value::Double(l + r as f64),
                        Value::Double(r) => Value::Double(l + r),
                        _ => return Err("`+`'s right operand can only support long & double when left operand is double".to_owned())
                    }
                    Value::String(mut l) => match r {
                        Value::Null => {
                            l.push_str("null");
                            Value::String(l)
                        }
                        Value::True => {
                            l.push_str("true");
                            Value::String(l)
                        }
                        Value::False => {
                            l.push_str("false");
                            Value::String(l)
                        }
                        Value::Bool(v) => {
                            l.push_str(if v {"true"} else {"false"});
                            Value::String(l)
                        }
                        Value::Long(v) => {
                            l.push_str(&v.to_string());
                            Value::String(l)
                        }
                        Value::Double(v) => {
                            l.push_str(&v.to_string());
                            Value::String(l)
                        }
                        Value::String(v) => {
                            l.push_str(&v);
                            Value::String(l)
                        }
                        Value::Class(c) => {
                            l.push_str(&format!("<class: {}>", unsafe {&(*c).name}));
                            Value::String(l)
                        }
                        Value::Instance(i) => {
                            l.push_str(&format!("<class: {}'s instance>", unsafe {&(*i.borrow().class).name}));
                            Value::String(l)
                        }
                        Value::Function(f) => {
                            l.push_str(&format!("<function: {}>", unsafe {&(*f).name}));
                            Value::String(l)
                        }
                        Value::Method(m) => {
                            l.push_str(&format!("<class: {}'s method: {}>", m.class_name(), m.name()));
                            Value::String(l)
                        }
                        Value::ForeignFunction(ff) => {
                            l.push_str(&format!("<foreign function: {}>", ff.name));
                            Value::String(l)
                        }
                    }
                    _ => return Err("`+` can only used between long, double and string".to_owned())
                };
                push_stack(frame, stack, res);
            }

            OP_SUB => bin_op!(frame, stack, -, "-"),
            OP_MUL => bin_op!(frame, stack, *, "*"),
            OP_DIV => bin_op!(frame, stack, /, "/"),

            OP_NOT => {
                let r = is_false(&pop_stack(frame, stack));
                push_stack(frame, stack, Value::Bool(r));
            }

            OP_CMP_EQ => {
                let r = pop_stack(frame, stack);
                let l = pop_stack(frame, stack);
                let res = if mem::discriminant(&l) != mem::discriminant(&r) {
                    false
                } else {
                    match l {
                        Value::Instance(l) => {
                            let r = unsafe {r.as_instance_unchecked()};
                            Rc::as_ptr(&l) == Rc::as_ptr(r)
                        }
                        _ => l == r
                    }
                };
                push_stack(frame, stack, Value::Bool(res));
            }
            OP_CMP_BANGEQ => {
                let r = pop_stack(frame, stack);
                let l = pop_stack(frame, stack);
                let res = if mem::discriminant(&l) != mem::discriminant(&r) {
                    true
                } else {
                    match l {
                        Value::Instance(l) => {
                            let r = unsafe {r.as_instance_unchecked()};
                            Rc::as_ptr(&l) != Rc::as_ptr(r)
                        }
                        _ => l != r
                    }
                };
                push_stack(frame, stack, Value::Bool(res));
            }

            OP_CMP_GT => cmp_op!(frame, stack, >),
            OP_CMP_LT => cmp_op!(frame, stack, <),
            OP_CMP_GTEQ => cmp_op!(frame, stack, >=),
            OP_CMP_LTEQ => cmp_op!(frame, stack, <=),

            OP_IF => {
                let idx = reader.next_u16()?;
                let v = pop_stack(frame, stack);
                if is_true(&v) {
                    reader.set_offset(idx as usize)?;
                }
            }
            OP_IF_NOT => {
                let idx = reader.next_u16()?;
                let v = pop_stack(frame, stack);
                if is_false(&v) {
                    reader.set_offset(idx as usize)?;
                }
            }
            OP_GOTO => {
                let idx = reader.next_u16()?;
                reader.set_offset(idx as usize)?;
            }

            OP_INVOKE => {
                let params = reader.next_u8()?;
                let owner = stack.read(frame.sp.get() as isize - params as isize - 1);
                match owner {
                    Value::Class(class) => {
                        // we don't have custom constructor now, so param count should be 0
                        if params != 0 {
                            return Err(format!("don't support pass arguments to class's constructor: {}", unsafe {&*class}.name));
                        }
                        pop_stack(frame, stack);// owner
                        push_stack(frame, stack, Value::Instance(Rc::new(RefCell::new(Instance::new(class)))));
                    }
                    Value::Function(func) => {
                        let new_frame = Frame::new(FrameType::Func(func));
                        let func = unsafe {&(*func)};
                        if func.params != params {
                            return Err(format!("function: {}'s param count: {}, but got: {params}", func.name, func.params));
                        }
                        let sp = frame.sp.get();
                        new_frame.sp.set(sp);
                        new_frame.sb.set(sp - params as usize);

                        frame.pc.set(reader.offset());
                        frame.sp.set(sp - params as usize - 1);// -1 the function owner

                        return Ok(Some(new_frame));
                    }
                    Value::Method(method) => {
                        let new_frame = Frame::new(FrameType::Method(method.method));
                        if method.param_count() != params {
                            return Err(format!("method: {}'s param count: {}, but got: {params}", method.name(), method.param_count()));
                        }
                        let sp = frame.sp.get();
                        stack.write(sp as isize, Value::Instance(method.instance.clone()));// this
                        new_frame.sp.set(sp + 1);
                        new_frame.sb.set(sp - params as usize);

                        frame.pc.set(reader.offset());
                        frame.sp.set(sp - params as usize - 1);// -1 the method owner

                        return Ok(Some(new_frame));
                    }
                    Value::ForeignFunction(ff) => {
                        if ff.params != params {
                            return Err(format!("foreign function: {}'s param count: {}, but got: {params}", ff.name, ff.params));
                        }
                        let mut args = VecDeque::with_capacity(params as usize);
                        for _ in 0 .. params {
                            args.push_front(pop_stack(frame, stack));
                        }
                        let sp = frame.sp.get();
                        frame.sp.set(sp - 1);// -1 the foreign function owner
                        let res = ff.entry.invoke(args);
                        push_stack(frame, stack, res);
                    }
                    _ => return Err("only class、function、method and foreign function can be invoked".to_owned())
                }
            }

            OP_RETURN => return Ok(None),

            OP_POP => {
                pop_stack(frame, stack);
            }

            OP_SET_GLOBAL => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(var)) = program.constant_pool.get(idx as usize) else {
                    return Err("`SET_GLOBAL` expect string argument as global variable name".to_owned());
                };
                let v = pop_stack(frame, stack);
                if matches!(v, Value::Method(_)) {
                    return Err("method can't assign to variable".to_owned());
                }
                globals.insert(var.to_owned(), v);
            }
            OP_GET_GLOBAL => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(var)) = program.constant_pool.get(idx as usize) else {
                    return Err("`GET_GLOBAL` expect string argument as global variable name".to_owned());
                };
                let Some(v) = globals.get(var) else {
                    return Err(format!("global variable: {var} used before define"));
                };
                push_stack(frame, stack, v.clone());
            }
            OP_SET_LOCAL => {
                let idx = reader.next_u8()?;
                let v = pop_stack(frame, stack);
                if matches!(v, Value::Method(_)) {
                    return Err("method can't assign to variable".to_owned());
                }
                stack.write(frame.sb.get() as isize + idx as isize, v);
            }
            OP_GET_LOCAL => {
                let idx = reader.next_u8()?;
                let v = stack.read(frame.sb.get() as isize + idx as isize);
                push_stack(frame, stack, v);
            }

            OP_SET_FIELD => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(var)) = program.constant_pool.get(idx as usize) else {
                    return Err("`SET_FIELD` expect string argument as field name".to_owned());
                };
                let v = pop_stack(frame, stack);
                if matches!(v, Value::Method(_)) {
                    return Err("method can't assign to class's field".to_owned());
                }
                let owner = pop_stack(frame, stack);
                match owner {
                    Value::Instance(instance) => {
                        if unsafe {(*instance.borrow().class).methods.contains_key(var)} {
                            return Err(format!("class: {} already has method named: {var}, can't assign new value to it"
                                               , unsafe {&(*instance.borrow().class).name}));
                        }
                        instance.borrow_mut().fields.insert(var.to_owned(), v);
                    }
                    _ => return Err("`SET_FIELD` owner should be class's instance".to_owned())
                }
            }

            OP_GET_MEMBER => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(name)) = program.constant_pool.get(idx as usize) else {
                    return Err("`GET_MEMBER` expect string argument as member name".to_owned());
                };
                let owner = pop_stack(frame, stack);
                match owner {
                    Value::Instance(instance) => {
                        let class = unsafe {&(*instance.borrow().class)};
                        let v = if let Some(method) = class.methods.get(name) {
                            Value::Method(MemMethod::new(instance.clone(), method as *const Method))
                        } else if let Some(v) = instance.borrow().fields.get(name) {
                            v.clone()
                        } else {
                            Value::Null
                        };
                        push_stack(frame, stack, v);
                    }
                    _ => return Err("`GET_MEMBER` owner should be class's instance".to_owned())
                }
            }
            _ => return Err(format!("unknown opcode: {opcode}"))
        }
    }

    Ok(None)
}


fn push_stack(frame: &Frame, stack: &Stack<Value>, value: Value) {
    let sp = frame.sp.get();
    stack.write(sp as isize, value);
    frame.sp.set(sp + 1);
}

fn pop_stack(frame: &Frame, stack: &Stack<Value>) -> Value {
    let mut sp = frame.sp.get();
    sp -= 1;
    frame.sp.set(sp);
    stack.read(sp as isize)
}

fn is_true(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::True => true,
        Value::False => false,
        Value::Bool(v) => *v,
        Value::Long(v) => *v != 0,
        Value::Double(v) => *v != 0f64,
        Value::String(s) => !s.is_empty(),
        Value::Class(_) => true,
        Value::Instance(_) => true,
        Value::Function(_) => true,
        Value::Method(_) => true,
        Value::ForeignFunction(_) => true
    }
}

fn is_false(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::True => false,
        Value::False => true,
        Value::Bool(v) => !v,
        Value::Long(v) => *v == 0,
        Value::Double(v) => *v == 0f64,
        Value::String(s) => s.is_empty(),
        Value::Class(_) => false,
        Value::Instance(_) => false,
        Value::Function(_) => false,
        Value::Method(_) => false,
        Value::ForeignFunction(_) => false
    }
}

fn print_error_and_exit(msg: &str, frames: &[Frame]) -> ! {
    eprintln!("Error:  {msg}");
    for frame in frames.iter().rev() {
        match &frame.frame_type {
            FrameType::Func(f) => {
                let name = unsafe {&(**f).name};
                if name != ENTRY_NAME {
                    eprintln!("      in function: {name}");
                }
            }
            FrameType::Method(m) => {
                let method = unsafe {&**m};
                eprintln!("      in method:  {}.{}", method.class_name, method.name);
            }
        }
    }
    exit(1);
}
