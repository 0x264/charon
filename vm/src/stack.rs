use std::ffi::{c_void, CString};
use std::marker::PhantomData;
use std::ptr;
use libc::size_t;

pub struct Stack<T> {
    ptr: *mut c_void,
    base: *mut T,
    _marker: PhantomData<T>
}

impl<T> Stack<T> {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let ptr = mem_map()?;
            MAPPED.push(ptr as usize);
            
            Ok(Self {
                ptr,
                base: ptr.offset(PAGE_SIZE as isize) as *mut T,
                _marker: PhantomData
            })
        }
    }
    
    pub fn read(&self, off: isize) -> T {
        unsafe {
            self.base.offset(off).read()
        }
    }
    
    pub fn write(&self, off: isize, value: T) {
        unsafe {
            self.base.offset(off).write(value);
        }
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr, MAP_SIZE);
            MAPPED.retain(|p| *p != self.ptr as usize);
        }
    }
}

const PAGE_SIZE: size_t = 4096;
const STACK_SIZE: size_t = PAGE_SIZE * 256;
const MAP_SIZE: size_t = STACK_SIZE + PAGE_SIZE * 2;// with guard page

static mut MAPPED: Vec<usize> = Vec::new();

unsafe fn mem_map() -> Result<*mut c_void, String> {
    let ptr = libc::mmap(ptr::null_mut(),
                                    MAP_SIZE,
                                    libc::PROT_NONE,
                                    libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                                    -1,
                                    0
    );
    if ptr == libc::MAP_FAILED {
        return Err(errno_msg());
    }
    
    if libc::mprotect(ptr.offset(PAGE_SIZE as isize), STACK_SIZE, libc::PROT_READ | libc::PROT_WRITE) == -1 {
        let errmsg = errno_msg();
        libc::munmap(ptr, MAP_SIZE);
        Err(errmsg)
    } else {
        Ok(ptr)
    }
}

unsafe fn errno_msg() -> String {
    CString::from_raw(libc::strerror(*libc::__error()))
        .into_string()
        .unwrap_or("unknown error".to_owned())
}