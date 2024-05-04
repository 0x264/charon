use common::constant::ConstantItem;
use common::program::{Class, Program};
use common::opcode::*;
use common::reader::LEReader;
use common::Result;

pub fn disassemble(program: &Program) -> Result<()> {
    println!("version: {}.{}\n", program.major, program.minor);

    println!("class count: {}\n", program.classes.len());
    for class in program.classes.values() {
        disassemble_class(class, &program.constant_pool)?;
        println!();
    }

    println!("function count: {}\n", program.functions.len());
    for func in program.functions.values() {
        println!("function name: {}, param count: {}", func.name, func.params);
        disassemble_code(&func.code, &program.constant_pool, false)?;
        println!();
    }
    Ok(())
}

fn disassemble_class(class: &Class, cp: &[ConstantItem]) -> Result<()> {
    println!("class name: {}, method count: {}", class.name, class.methods.len());

    for method in class.methods.values() {
        println!("    method name: {}, param count: {}", method.name, method.params);
        disassemble_code(&method.code, cp, true)?;
        println!();
    }
    Ok(())
}

fn disassemble_code(code: &[u8], cp: &[ConstantItem], intent: bool) -> Result<()> {
    let mut reader = LEReader::new(code);
    let mut codeinfo = CodeInfo::new();

    while let Ok(opcode) = reader.next_u8() {
        codeinfo.add_line_byteoff((reader.offset() - 1) as u16);

        let inst = match opcode {
            OP_CONST_NULL => new_plain_inst("CONST_NULL"),
            OP_CONST_TRUE => new_plain_inst("CONST_TRUE"),
            OP_CONST_FALSE => new_plain_inst("CONST_FALSE"),
            
            OP_LCONST_M1 => new_plain_inst("LCONST_M1"),
            OP_LCONST_0 => new_plain_inst("LCONST_0"),
            OP_LCONST_1 => new_plain_inst("LCONST_1"),
            OP_LCONST_2 => new_plain_inst("LCONST_2"),
            OP_LCONST_3 => new_plain_inst("LCONST_3"),
            OP_LCONST_4 => new_plain_inst("LCONST_4"),
            OP_LCONST_5 => new_plain_inst("LCONST_5"),
            
            OP_LDC => {
                let idx = reader.next_u16()?;
                let mut raw = format!("LDC {idx}    //");
                let Some(item) = cp.get(idx as usize) else {
                    return Err(format!("`op_ldc`'s argument: {idx} not found in constant pool"));
                };
                match item {
                    ConstantItem::Long(v) => {
                        raw.push_str("Long: ");
                        raw.push_str(&v.to_string());
                    }
                    ConstantItem::Double(v) => {
                        raw.push_str("Double: ");
                        raw.push_str(&v.to_string());
                    }
                    ConstantItem::String(v) => {
                        raw.push_str("String: ");
                        raw.push_str(v);
                    }
                }
                InstInfo::Plain(raw)
            }
            
            OP_NEG => new_plain_inst("NEG"),
            OP_ADD => new_plain_inst("ADD"),
            OP_SUB => new_plain_inst("SUB"),
            OP_MUL => new_plain_inst("MUL"),
            OP_DIV => new_plain_inst("DIV"),
            
            OP_NOT => new_plain_inst("NOT"),
            
            OP_CMP_EQ => new_plain_inst("CMP_EQ"),
            OP_CMP_BANGEQ => new_plain_inst("CMP_BANGEQ"),
            OP_CMP_GT => new_plain_inst("CMP_GT"),
            OP_CMP_LT => new_plain_inst("CMP_LT"),
            OP_CMP_GTEQ => new_plain_inst("CMP_GTEQ"),
            OP_CMP_LTEQ => new_plain_inst("CMP_LTEQ"),
            
            OP_IF => {
                let idx = reader.next_u16()?;
                InstInfo::Jump(format!("IF  {idx}"), idx)
            }
            OP_IF_NOT => {
                let idx = reader.next_u16()?;
                InstInfo::Jump(format!("IF_NOT  {idx}"), idx)
            }
            OP_GOTO => {
                let idx = reader.next_u16()?;
                InstInfo::Jump(format!("GOTO  {idx}"), idx)
            }
            
            OP_INVOKE => InstInfo::Plain(format!("INVOKE  // param count: {}", reader.next_u8()?)),
            
            OP_RETURN => new_plain_inst("RETURN"),
            OP_POP => new_plain_inst("POP"),

            OP_DEF_GLOBAL => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(arg)) = cp.get(idx as usize) else {
                    return Err("`DEF_GLOBAL` expect string argument as arg name".to_owned());
                };
                InstInfo::Plain(format!("DEF_GLOBAL  {idx}    // {arg}"))
            }
            
            OP_SET_GLOBAL => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(arg)) = cp.get(idx as usize) else {
                    return Err("`SET_GLOBAL` expect string argument as arg name".to_owned());
                };
                InstInfo::Plain(format!("SET_GLOBAL  {idx}    // {arg}"))
            }

            OP_GET_GLOBAL => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(arg)) = cp.get(idx as usize) else {
                    return Err("`GET_GLOBAL` expect string argument as arg name".to_owned());
                };
                InstInfo::Plain(format!("GET_GLOBAL  {idx}    // {arg}"))
            }
            
            OP_SET_LOCAL => InstInfo::Plain(format!("SET_LOCAL  {}", reader.next_u8()?)),
            OP_GET_LOCAL => InstInfo::Plain(format!("GET_LOCAL  {}", reader.next_u8()?)),
            
            OP_SET_FIELD => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(arg)) = cp.get(idx as usize) else {
                    return Err("`SET_FIELD` expect string argument as arg name".to_owned());
                };
                InstInfo::Plain(format!("SET_FIELD  {idx}    // {arg}"))
            }

            OP_GET_MEMBER => {
                let idx = reader.next_u16()?;
                let Some(ConstantItem::String(arg)) = cp.get(idx as usize) else {
                    return Err("`GET_MEMBER` expect string argument as arg name".to_owned());
                };
                InstInfo::Plain(format!("GET_MEMBER  {idx}    // {arg}"))
            }
            
            OP_DUP => new_plain_inst("DUP"),
            
            _ => return Err(format!("unknown opcode: {opcode}"))
        };
        codeinfo.add_inst(inst);
    }
    
    for (idx, info) in codeinfo.insts.iter().enumerate() {
        if intent {
            print!("    ");
        }
        
        print!("{idx:>4}:  ");
        match info {
            InstInfo::Plain(s) => println!("{s}"),
            InstInfo::Jump(s, off) => {
                print!("{s}    // jump to: ");
                let mut found = false;
                for (line, byteoff) in codeinfo.line_byteoff.iter().enumerate() {
                    if off == byteoff {
                        println!("{line}");
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(format!("jump byte offset: {off} error, in {s}"));
                }
            }
        }
    }

    Ok(())
}

enum InstInfo {
    Plain(String),
    Jump(String, u16)// with jump offset in code byte array
}

fn new_plain_inst(assembly: &str) -> InstInfo {
    InstInfo::Plain(assembly.to_owned())
}

struct CodeInfo {
    insts: Vec<InstInfo>,
    line_byteoff: Vec<u16> // index is assembly code line, value is start offset in code byte array
}

impl CodeInfo {
    fn new() -> Self {
        Self {
            insts: Vec::new(),
            line_byteoff: Vec::new()
        }
    }

    fn add_inst(&mut self, inst: InstInfo) {
        self.insts.push(inst)
    }

    fn add_line_byteoff(&mut self, byteoff: u16) {
        self.line_byteoff.push(byteoff);
    }
}
