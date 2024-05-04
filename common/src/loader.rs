use ahash::{HashMap, HashMapExt};
use crate::constant::*;
use crate::program::{Class, Function, Method, Program};
use crate::reader::LEReader;
use crate::Result;

pub struct Loader<'a> {
    reader: LEReader<'a>,
    cp: Vec<ConstantItem>,
    classes: HashMap<String, Class>,
    functions: HashMap<String, Function>
}

impl Loader<'_> {
    pub fn new(bytes: &[u8]) -> Loader {
        Loader {
            reader: LEReader::new(bytes),
            cp: Vec::new(),
            classes: HashMap::new(),
            functions: HashMap::new()
        }
    }

    pub fn load(mut self) -> Result<Program> {
        for magic in MAGIC.as_bytes() {
            if *magic != self.reader.next_u8()? {
                return Err("input is not charon bytecode, magic not match".to_owned());
            }
        }

        let minor = self.reader.next_u8()?;
        let major = self.reader.next_u8()?;
        if major > CURRENT_VERSION_MAJOR
            || (major == CURRENT_VERSION_MAJOR && minor > CURRENT_VERSION_MINOR) {
            return Err(format!("unsupport version: {major}.{minor}"));
        }

        self.load_constant_pool()?;
        self.load_classes()?;
        self.load_functions()?;

        Ok(Program::new(minor, major, self.cp, self.classes, self.functions))
    }

    fn load_constant_pool(&mut self) -> Result<()> {
        let pool_item_count = self.reader.next_u16()? as usize;
        self.cp.reserve(pool_item_count);

        for _ in 0 .. pool_item_count {
            let item = match self.reader.next_u8()? {
                CONSTANT_LONG => ConstantItem::Long(self.reader.next_u64()? as i64),
                CONSTANT_DOUBLE => ConstantItem::Double(f64::from_bits(self.reader.next_u64()?)),
                CONSTANT_STRING => {
                    let len = self.reader.next_u16()? as usize;
                    let mut v = Vec::with_capacity(len);
                    self.reader.read_to(&mut v, len)?;
                    ConstantItem::String(String::from_utf8(v)
                        .map_err(|_| "constant string is not valid utf-8 format".to_owned())?)
                }
                other => return Err(format!("unknown constant pool tag: {other}"))
            };
            self.cp.push(item);
        }

        Ok(())
    }

    fn load_classes(&mut self) -> Result<()> {
        let class_count = self.reader.next_u16()?;
        for _ in 0 .. class_count {
            let class = self.load_class()?;
            self.classes.insert(class.name.clone(), class);
        }

        Ok(())
    }

    fn load_class(&mut self) -> Result<Class> {
        let name_index = self.reader.next_u16()?;
        let class_name = self.load_string_constant(name_index)?;
        
        let method_count = self.reader.next_u16()? as usize;
        let mut methods = HashMap::with_capacity(method_count);
        for _ in 0 .. method_count {
            let method = self.load_method(&class_name)?;
            methods.insert(method.name.clone(), method);
        }
        
        Ok(Class::new(class_name, methods))
    }

    fn load_method(&mut self, class_name: &str) -> Result<Method> {
        let Function {name, params, max_locals, code} = self.load_function()?;
        Ok(Method::new(class_name.to_owned(), name, params, max_locals, code))
    }
    
    fn load_functions(&mut self) -> Result<()> {
        let func_count = self.reader.next_u16()? as usize;
        self.functions.reserve(func_count);
        for _ in 0 .. func_count {
            let func = self.load_function()?;
            self.functions.insert(func.name.clone(), func);
        }
        Ok(())
    }
    
    fn load_function(&mut self) -> Result<Function> {
        let name_idx = self.reader.next_u16()?;
        let name = self.load_string_constant(name_idx)?;
        let params = self.reader.next_u8()?;
        let max_locals = self.reader.next_u8()?;
        let code_len = self.reader.next_u16()? as usize;
        let mut code = Vec::with_capacity(code_len);
        self.reader.read_to(&mut code, code_len)?;
        Ok(Function::new(name, params, max_locals, code))
    }
    
    fn load_string_constant(&mut self, idx: u16) -> Result<String> {
        let Some(cp_item) = self.cp.get(idx as usize) else {
            return Err(format!("no constant pool item exists for index: {idx}"));
        };
        
        match cp_item {
            ConstantItem::Long(_) => Err("index ref to type Long".to_owned()),
            ConstantItem::Double(_) => Err("index ref to type Double".to_owned()),
            ConstantItem::String(s) => Ok(s.clone())
        }
    }
}