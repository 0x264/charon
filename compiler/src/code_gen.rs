use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use crate::ast::{AssignOp, BinaryOp, ClassDecl, Expr, FuncDecl, LogicOp, Program, Stmt, UnaryOp};
use common::constant::*;
use common::opcode::*;

struct ConstantPool {
    long: HashMap<i64, u16>,
    double: HashMap<u64, u16>, // key is double's int bits
    string: HashMap<String, u16>,
    count: u16,
    code: Vec<u8>
}

impl ConstantPool {
    fn new() -> Self {
        Self {
            long: HashMap::new(),
            double: HashMap::new(),
            string: HashMap::new(),
            count: 0,
            code: Vec::new()
        }
    }

    fn const_long(&mut self, v: i64) -> u16 {
        *self.long.entry(v).or_insert_with(|| {
            self.code.push(CONSTANT_LONG);
            self.code.extend_from_slice(&v.to_le_bytes());

            let idx = self.count;
            self.count += 1;
            idx
        })
    }

    fn const_double(&mut self, v: f64) -> u16 {
        *self.double.entry(v.to_bits()).or_insert_with(|| {
            self.code.push(CONSTANT_DOUBLE);
            self.code.extend_from_slice(&v.to_bits().to_le_bytes());

            let idx = self.count;
            self.count += 1;
            idx
        })
    }

    fn const_string(&mut self, v: &str) -> u16 {
        if let Some(idx) = self.string.get(v) {
            return *idx;
        }

        self.code.push(CONSTANT_STRING);
        self.code.extend_from_slice(&(v.len() as u16).to_le_bytes());
        self.code.extend_from_slice(v.as_bytes());

        let idx = self.count;
        self.count += 1;
        self.string.insert(v.to_owned(), idx);
        idx
    }
}

#[derive(PartialEq)]
enum CallableType {
    Method(u8),// param count(not include 'this')
    Func,
    None
}

struct Context {
    local_var: HashMap<String, u8>,
    count: u8,

    callable_type: CallableType,

    loop_start_pos: Option<u16>,
    loop_out_patch_pos: Vec<u16>
}

impl Context {
    fn new(callable_type: CallableType) -> Self {
        Self {
            local_var: HashMap::new(),
            count: 0,
            callable_type,
            loop_start_pos: None,
            loop_out_patch_pos: Vec::new()
        }
    }

    // we allow redefine variable
    fn define_var(&mut self, var: &str) -> u8 {
        if let Some(idx) = self.local_var.get(var) {
            return *idx;
        }

        let idx = self.count;
        self.count += 1;
        self.local_var.insert(var.to_owned(), idx);
        idx
    }

    fn get_var(&mut self, var: &str) -> Option<u8> {
        self.local_var.get(var).copied()
    }
}

type Result<T> = std::result::Result<T, String>;// use string as error type, ignore line, column info

pub fn check_and_gen(program: &Program) -> Result<Vec<u8>> {
    let mut global = HashSet::new();
    let mut cp = ConstantPool::new();

    // contains classes & functions info (not include header & constant pool info)
    let mut code = Vec::new();
    code.extend_from_slice(&(program.classes.len() as u16).to_le_bytes());
    for class in &program.classes {
        if global.contains(&class.name) {
            return Err(format!("multi class with name: {} found", class.name));
        }
        global.insert(class.name.clone());
        gen_class(class, &mut cp, &mut code)?;
    }

    code.extend_from_slice(&(program.funcs.len() as u16).to_le_bytes());

    for func in &program.funcs {
        let mut context = if func.name == ENTRY_NAME {
            Context::new(CallableType::None)
        } else {
            Context::new(CallableType::Func)
        };
        gen_func(func, &mut context, &mut cp, &mut code)?;
    }
    
    let mut bytes = Vec::with_capacity(20 + cp.code.len() + code.len());
    bytes.extend_from_slice(MAGIC.as_bytes());
    bytes.push(CURRENT_VERSION_MINOR);
    bytes.push(CURRENT_VERSION_MAJOR);
    bytes.extend_from_slice(&cp.count.to_le_bytes());
    bytes.extend_from_slice(&cp.code);
    bytes.extend_from_slice(&code);

    Ok(bytes)
}


fn gen_class(class: &ClassDecl, cp: &mut ConstantPool, code: &mut Vec<u8>) -> Result<()> {
    // class name's index
    code.extend_from_slice(&cp.const_string(&class.name).to_le_bytes());

    // method count
    code.extend_from_slice(&(class.methods.len() as u16).to_le_bytes());

    let mut method_names = HashSet::with_capacity(class.methods.len());
    for method in &class.methods {
        if method_names.contains(&method.name) {
            return Err(format!("multi method with name: {} in class: {}", method.name, class.name));
        }
        method_names.insert(method.name.clone());

        let mut context = Context::new(CallableType::Method(method.params.len() as u8));
        gen_func(method, &mut context, cp, code)?;
    }

    Ok(())
}

fn gen_func(func: &FuncDecl, context: &mut Context, cp: &mut ConstantPool, code: &mut Vec<u8>) -> Result<()> {
    // name index
    code.extend_from_slice(&cp.const_string(&func.name).to_le_bytes());

    // param count
    code.push(func.params.len() as u8);

    for arg in &func.params {
        context.define_var(arg);
    }

    // define 'this' as the last arg
    if let CallableType::Method(_) = context.callable_type {
        context.define_var("this");
    }

    let mut body = Vec::new();
    for stmt in &func.body {
        gen_stmt(stmt, context, cp, &mut body)?;
    }

    // default return null
    body.push(OP_CONST_NULL);
    body.push(OP_RETURN);

    code.extend_from_slice(&(body.len() as u16).to_le_bytes());
    code.extend_from_slice(&body);
    Ok(())
}

fn gen_stmt(stmt: &Stmt, context: &mut Context, cp: &mut ConstantPool, code: &mut Vec<u8>) -> Result<()> {
    match stmt {
        Stmt::VarDef(vardef) => {
            if let Some(init) = &vardef.init {
                gen_expr(init, context, cp, code)?;
            } else {
                code.push(OP_CONST_NULL);
            }
            code.push(OP_SET_LOCAL);
            code.push(context.define_var(&vardef.name));
        }
        Stmt::Expr(e) => {
            match e.as_ref() {
                Expr::True | Expr::Flase | Expr::Null |
                Expr::Long(_) | Expr::Double(_) | Expr::String(_) => (),
                Expr::This => if !matches!(context.callable_type, CallableType::Method(_)) {
                    return Err("`this` can only used in methods".to_owned());
                }
                _ => {
                    gen_expr(e, context, cp, code)?;
                    code.push(OP_POP);
                }
            }
        }
        Stmt::SetVar(setvar) => {
            let opcode;
            let idx: u16;
            if let Some(local) = context.get_var(&setvar.to) {
                opcode = OP_SET_LOCAL;
                idx = local as u16;
            } else {
                opcode = OP_SET_GLOBAL;
                idx = cp.const_string(&setvar.to);
            }

            if setvar.op == AssignOp::Assign {
                gen_expr(&setvar.value, context, cp, code)?;
            } else {
                if opcode == OP_SET_LOCAL {
                    code.push(OP_GET_LOCAL);
                    code.push(idx as u8);
                } else {
                    code.push(OP_GET_GLOBAL);
                    code.extend_from_slice(&idx.to_le_bytes());
                }

                gen_expr(&setvar.value, context, cp, code)?;

                let op = match setvar.op {
                    AssignOp::AddAssign => OP_ADD,
                    AssignOp::SubAssign => OP_SUB,
                    AssignOp::MultiplyAssign => OP_MUL,
                    AssignOp::DivideAssign => OP_DIV,
                    AssignOp::Assign => unreachable!(),
                };
                code.push(op);
            }

            code.push(opcode);
            if opcode == OP_SET_LOCAL {
                code.push(idx as u8);
            } else {
                code.extend_from_slice(&idx.to_le_bytes());
            }
        }
        Stmt::Setter(setter) => {
            gen_expr(&setter.owner, context, cp, code)?;
            let idx = cp.const_string(&setter.field);
            if setter.op == AssignOp::Assign {
                gen_expr(&setter.value, context, cp, code)?;
            } else {
                code.push(OP_DUP);
                code.push(OP_GET_MEMBER);
                code.extend_from_slice(&idx.to_le_bytes());
                gen_expr(&setter.value, context, cp, code)?;
                let op = match setter.op {
                    AssignOp::AddAssign => OP_ADD,
                    AssignOp::SubAssign => OP_SUB,
                    AssignOp::MultiplyAssign => OP_MUL,
                    AssignOp::DivideAssign => OP_DIV,
                    AssignOp::Assign => unreachable!(),
                };
                code.push(op);
            }
            code.push(OP_SET_FIELD);
            code.extend_from_slice(&idx.to_le_bytes());
        }
        Stmt::If(ifstmt) => {
            gen_expr(&ifstmt.cond, context, cp, code)?;
            code.push(OP_IF_NOT);
            let off = code.len() as u16;
            code.push(0);code.push(0);

            for stmt in &ifstmt.then {
                gen_stmt(stmt, context, cp, code)?;
            }

            let target;
            if ifstmt.els.is_empty() {
                target = code.len() as u16;
            } else {
                code.push(OP_GOTO);
                let off2 = code.len() as u16;
                code.push(0);code.push(0);

                target = code.len() as u16;
                for stmt in &ifstmt.els {
                    gen_stmt(stmt, context, cp, code)?;
                }
                patch(code, off2, code.len() as u16);
            }
            patch(code, off, target);
        }
        Stmt::While(while_stmt) => {
            let loop_back = code.len() as u16;
            gen_expr(&while_stmt.cond, context, cp, code)?;
            code.push(OP_IF_NOT);
            let off = code.len() as u16;
            code.push(0);code.push(0);

            context.loop_start_pos = Some(loop_back);
            for stmt in &while_stmt.body {
                gen_stmt(stmt, context, cp, code)?;
            }
            context.loop_start_pos = None;

            // jump back
            code.push(OP_GOTO);
            code.extend_from_slice(&loop_back.to_le_bytes());

            let while_end = code.len() as u16;
            patch(code, off, while_end);

            for patch_pos in &context.loop_out_patch_pos {
                patch(code, *patch_pos, while_end);
            }
            context.loop_out_patch_pos.clear();
        }
        Stmt::Break => {
            if context.loop_start_pos.is_none() {
                return Err("`break` can only used in while loop".to_owned());
            }
            code.push(OP_GOTO);
            context.loop_out_patch_pos.push(code.len() as u16);
            code.push(0);code.push(0);
        }
        Stmt::Continue => {
            let Some(loop_back) = context.loop_start_pos else {
                return Err("`continue` can only used in while loop".to_owned());
            };
            code.push(OP_GOTO);
            code.extend_from_slice(&loop_back.to_le_bytes());
        }
        Stmt::Return(ret) => {
            if context.callable_type == CallableType::None {
                return Err("`return` can only used in function or method".to_owned());
            }
            match ret {
                None => code.push(OP_CONST_NULL),
                Some(value) => gen_expr(value, context, cp, code)?
            }
            code.push(OP_RETURN);
        }
        Stmt::Block(block) => {
            for stmt in block {
                gen_stmt(stmt, context, cp, code)?;
            }
        }
    }

    Ok(())
}

fn gen_expr(expr: &Expr, context: &mut Context, cp: &mut ConstantPool, code: &mut Vec<u8>) -> Result<()> {
    match expr {
        Expr::True => code.push(OP_CONST_TRUE),
        Expr::Flase => code.push(OP_CONST_FALSE),
        Expr::Null => code.push(OP_CONST_NULL),
        Expr::This => {
            let CallableType::Method(count) = context.callable_type else {
                return Err("`this` can only used in methods".to_owned());
            };
            code.push(OP_GET_LOCAL);
            code.push(count);
        }
        Expr::Long(v) => {
            let opcode = match *v {
                -1 => OP_LCONST_M1,
                0  => OP_LCONST_0,
                1  => OP_LCONST_1,
                2  => OP_LCONST_2,
                3  => OP_LCONST_3,
                4  => OP_LCONST_4,
                5  => OP_LCONST_5,
                _  => {
                    code.push(OP_LDC);
                    code.extend_from_slice(&cp.const_long(*v).to_le_bytes());
                    return Ok(());
                }
            };
            code.push(opcode);
        }
        Expr::Double(v) => {
            code.push(OP_LDC);
            code.extend_from_slice(&cp.const_double(*v).to_le_bytes());
        }
        Expr::String(s) => {
            code.push(OP_LDC);
            code.extend_from_slice(&cp.const_string(s).to_le_bytes());
        }
        Expr::Binary(binary) => {
            gen_expr(&binary.left, context, cp, code)?;
            gen_expr(&binary.right, context, cp, code)?;
            let opcode = match binary.op {
                BinaryOp::Add => OP_ADD,
                BinaryOp::Sub => OP_SUB,
                BinaryOp::Multiply => OP_MUL,
                BinaryOp::Divide => OP_DIV,
                BinaryOp::Gt => OP_CMP_GT,
                BinaryOp::Lt => OP_CMP_LT,
                BinaryOp::EqEq => OP_CMP_EQ,
                BinaryOp::GtEq => OP_CMP_GTEQ,
                BinaryOp::LtEq => OP_CMP_LTEQ,
                BinaryOp::BangEq => OP_CMP_BANGEQ
            };
            code.push(opcode);
        }
        Expr::Logic(logic) => {
            match logic.op {
                LogicOp::And => {
                    gen_expr(&logic.left, context, cp, code)?;
                    code.push(OP_IF_NOT);
                    let off = code.len() as u16;
                    code.push(0);code.push(0);
                    gen_expr(&logic.right, context, cp, code)?;
                    code.push(OP_IF_NOT);
                    let off2 = code.len() as u16;
                    code.push(0);code.push(0);
                    code.push(OP_CONST_TRUE);
                    let target = code.len() as u16;
                    code.push(OP_CONST_FALSE);
                    patch(code, off, target);
                    patch(code, off2, target);
                }
                LogicOp::Or => {
                    gen_expr(&logic.left, context, cp, code)?;
                    code.push(OP_IF);
                    let off = code.len() as u16;
                    code.push(0);code.push(0);
                    gen_expr(&logic.right, context, cp, code)?;
                    code.push(OP_IF);
                    let off2 = code.len() as u16;
                    code.push(0);code.push(0);
                    code.push(OP_CONST_FALSE);
                    let target = code.len() as u16;
                    code.push(OP_CONST_TRUE);
                    patch(code, off, target);
                    patch(code, off2, target);
                }
            }
        }
        Expr::Unary(unary) => {
            gen_expr(&unary.expr, context, cp, code)?;
            let opcode = match unary.op {
                UnaryOp::Bang => OP_NOT,
                UnaryOp::Neg => OP_NEG
            };
            code.push(opcode);
        }
        Expr::Call(call) => {
            gen_expr(&call.owner, context, cp, code)?;
            for arg in &call.args {
                gen_expr(arg, context, cp, code)?;
            }
            code.push(OP_INVOKE);
            code.push(call.args.len() as u8);
        }
        Expr::GetVar(getvar) => {
            if let Some(idx) = context.local_var.get(getvar) {
                code.push(OP_GET_LOCAL);
                code.push(*idx);
            } else {
                code.push(OP_GET_GLOBAL);
                code.extend_from_slice(&cp.const_string(getvar).to_le_bytes());
            }
        }
        Expr::Getter(getter) => {
            gen_expr(&getter.owner, context, cp, code)?;
            code.push(OP_GET_MEMBER);
            code.extend_from_slice(&cp.const_string(&getter.member).to_le_bytes());
        }
    }
    Ok(())
}

fn patch(code: &mut Vec<u8>, to: u16, value: u16) {
    let bytes = value.to_le_bytes();
    unsafe {
        code.as_mut_ptr().offset(to as isize).write(*bytes.get_unchecked(0));
        code.as_mut_ptr().offset(to as isize + 1).write(*bytes.get_unchecked(1));
    }
}