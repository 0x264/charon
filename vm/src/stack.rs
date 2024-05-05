use std::ffi::{c_void, CString};
use std::marker::PhantomData;
use std::{mem, ptr};
use std::process::exit;
use std::ptr::{addr_of, addr_of_mut};
use libc::{c_int, SA_SIGINFO, sigaction, sighandler_t, siginfo_t, sigset_t, size_t};
use common::err_println;

pub struct Stack<T> {
    ptr: *mut c_void,
    base: *mut T,
    _marker: PhantomData<T>
}

impl<T> Stack<T> {
    pub fn new() -> Result<Self, String> {
        #[allow(clippy::ptr_offset_with_cast)] 
        unsafe {
            let ptr = mem_map()?;
            MAPPED.push(ptr as usize);

            if let Err(msg) = install_sigaction_for_segv() {
                err_println(&format!("failed to install sigaction for SIGSEGV: {msg}"));
            }
            
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

    #[allow(clippy::ptr_offset_with_cast)]
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

static mut OLD_SIGSEGV_SIGACTION: sigaction = sigaction {
    sa_sigaction: 0,
    sa_mask: 0,
    sa_flags: 0
};

static mut OLD_SIGBUS_SIGACTION: sigaction = sigaction {
    sa_sigaction: 0,
    sa_mask: 0,
    sa_flags: 0
};

unsafe extern "C" fn sig_handler(signum: c_int, siginfo: *const siginfo_t, context: *const c_void) {
    handle_out_of_stack(siginfo);

    let action = if signum == libc::SIGSEGV {
        OLD_SIGSEGV_SIGACTION
    } else if signum == libc::SIGBUS {
        OLD_SIGBUS_SIGACTION
    } else {
        unreachable!()
    };
    
    if action.sa_sigaction != 0 {
        let mut pre_mask: sigset_t = 0;
        libc::sigprocmask(libc::SIG_BLOCK, addr_of!(action.sa_mask), addr_of_mut!(pre_mask));
        let handler: extern "C" fn(c_int, *const siginfo_t, *const c_void) = mem::transmute(addr_of!(action.sa_sigaction));
        handler(signum, siginfo, context);
        libc::sigprocmask(libc::SIG_SETMASK, addr_of!(pre_mask), ptr::null_mut());
    } else {
        exit(1);
    }
}

unsafe fn handle_out_of_stack(siginfo: *const siginfo_t) {
    let Some(siginfo) = siginfo.as_ref() else {
        return;
    };
    
    let fault_addr = siginfo.si_addr as usize;
    #[allow(static_mut_refs)]
    for addr in &MAPPED {
        let addr = *addr;
        if fault_addr >= addr && fault_addr < addr + PAGE_SIZE {
            err_println("stack underflow");
            if let Some(notifier) = &STACK_ERROR_NOTIFIER {
                notifier.on_error();
            }
            exit(1);
        }
        
        if fault_addr >= addr + STACK_SIZE + PAGE_SIZE
            && fault_addr < addr + STACK_SIZE + PAGE_SIZE * 2 {
            err_println("stack overflow");
            if let Some(notifier) = &STACK_ERROR_NOTIFIER {
                notifier.on_error();
            }
            exit(1);
        }
    }
}

static mut HAS_INSTALLED_SIGSEGV_HANDLER: bool = false;
static mut HAS_INSTALLED_SIGBUS_HANDLER: bool = false;

unsafe fn install_sigaction_for_segv() -> Result<(), String> {
    if HAS_INSTALLED_SIGSEGV_HANDLER && HAS_INSTALLED_SIGBUS_HANDLER {
        return Ok(());
    }
    let mut action = sigaction {
        sa_sigaction: sig_handler as sighandler_t,
        sa_flags: SA_SIGINFO,
        sa_mask: 0
    };
    
    libc::sigemptyset(addr_of_mut!(action.sa_mask));// ignore error
    
    if !HAS_INSTALLED_SIGSEGV_HANDLER {
        if libc::sigaction(libc::SIGSEGV, addr_of!(action), addr_of_mut!(OLD_SIGSEGV_SIGACTION)) == -1 {
            return Err(errno_msg());
        }
        HAS_INSTALLED_SIGSEGV_HANDLER = true;
    }
    
    if !HAS_INSTALLED_SIGBUS_HANDLER {
        if libc::sigaction(libc::SIGBUS, addr_of!(action), addr_of_mut!(OLD_SIGBUS_SIGACTION)) == -1 {
            return Err(errno_msg());
        }
        HAS_INSTALLED_SIGBUS_HANDLER = true;
    }
        
    Ok(())
}

pub trait StackError {
    fn on_error(&self);
}

pub static mut STACK_ERROR_NOTIFIER: Option<Box<dyn StackError>> = None;