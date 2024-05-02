use std::ffi::{c_void, CString};
use std::marker::PhantomData;
use std::{mem, ptr};
use std::process::exit;
use libc::{c_int, SA_SIGINFO, sigaction, sighandler_t, siginfo_t, sigset_t, size_t};

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

            if let Err(msg) = install_sigaction_for_segv() {
                eprintln!("failed to install sigaction for SIGSEGV: {msg}");
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

static mut OLD_SIGACTION: sigaction = sigaction {
    sa_sigaction: 0,
    sa_mask: 0,
    sa_flags: 0
};

unsafe extern "C" fn sig_handler(signum: c_int, siginfo: *const siginfo_t, context: *const c_void) {
    handle_out_of_stack(siginfo);

    if OLD_SIGACTION.sa_sigaction != 0 {
        let mut pre_mask: sigset_t = 0;
        libc::sigprocmask(libc::SIG_BLOCK, OLD_SIGACTION.sa_mask as *const sigset_t, &mut pre_mask as *mut sigset_t);
        let handler: extern "C" fn(c_int, *const siginfo_t, *const c_void) = mem::transmute(OLD_SIGACTION.sa_sigaction as *const c_void);
        handler(signum, siginfo, context);
        libc::sigprocmask(libc::SIG_SETMASK, &pre_mask as *const sigset_t, ptr::null_mut());
    } else {
        exit(1);
    }
}

unsafe fn handle_out_of_stack(siginfo: *const siginfo_t) {
    let Some(siginfo) = siginfo.as_ref() else {
        return;
    };
    
    let fault_addr = siginfo.si_addr as usize;
    for addr in &MAPPED {
        let addr = *addr;
        if fault_addr >= addr && fault_addr < addr + PAGE_SIZE {
            eprintln!("stack underflow");
            exit(1);
        }
        
        if fault_addr >= addr + STACK_SIZE + PAGE_SIZE
            && fault_addr < addr + STACK_SIZE + PAGE_SIZE * 2 {
            eprintln!("stack overflow");
            exit(1);
        }
    }
}

static mut HAS_INSTALLED_HANDLER: bool = false;

unsafe fn install_sigaction_for_segv() -> Result<(), String> {
    if HAS_INSTALLED_HANDLER {
        return Ok(());
    }
    let mut action = sigaction {
        sa_sigaction: sig_handler as sighandler_t,
        sa_flags: SA_SIGINFO,
        sa_mask: 0
    };
    
    libc::sigemptyset(&mut action.sa_mask as *mut sigset_t);// ignore error
    if libc::sigaction(libc::SIGSEGV, &action as *const sigaction, &mut OLD_SIGACTION as *mut sigaction) == -1 {
        Err(errno_msg())
    } else {
        HAS_INSTALLED_HANDLER = true;
        Ok(())
    }
}