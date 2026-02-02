#![no_std]
#![no_main]

use alloc::vec;

use crate::{
    allocator::KernelAlloc,
    vfs::{FdTable, Vfs},
};

pub mod allocator;
pub mod drivers;
pub mod vfs;

extern crate alloc;

pub type Fd = i32;
pub type Pid = i32;

pub type ResultCode = i32;

pub const OK: ResultCode = 0;
pub const ERR: ResultCode = -1;

unsafe extern "C" {
    static __heap_base: u8;
}

#[link(wasm_import_module = "sys")]
unsafe extern "C" {
    pub fn serial_write(ptr: *const u8, len: usize) -> ResultCode;
}

#[link(wasm_import_module = "mem_ops")]
unsafe extern "C" {
    pub fn cp_from_bin(pid: Pid, other_ptr: *const u8, ptr: *mut u8, len: usize);
    pub fn cp_to_bin(pid: Pid, other_ptr: *mut u8, ptr: *const u8, len: usize);
}

pub fn print(text: &[u8]) {
    unsafe {
        serial_write(text.as_ptr(), text.len());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {}

struct Process {
    pid: Pid,
    fd_table: FdTable,
}

impl Process {
    const fn new(pid: Pid) -> Self {
        Self {
            pid,
            fd_table: FdTable::new(),
        }
    }
}

struct Kernel {
    vfs: Vfs,
    processes: [Option<Process>; 8],
}

impl Kernel {
    const fn new() -> Self {
        Self {
            vfs: Vfs::new(),
            processes: [const { None }; 8],
        }
    }

    fn get_or_create_process(&mut self, pid: Pid) -> Option<&mut Process> {
        let mut first_empty_idx = None;

        for (i, slot) in self.processes.iter().enumerate() {
            match slot {
                Some(p) if p.pid == pid => return self.processes[i].as_mut(),
                None if first_empty_idx.is_none() => first_empty_idx = Some(i),
                _ => {}
            }
        }

        if let Some(idx) = first_empty_idx {
            self.processes[idx] = Some(Process::new(pid));
            return self.processes[idx].as_mut();
        }

        None
    }
}

static mut KERNEL: Kernel = Kernel::new();

const SYS_READ: i32 = 0;
const SYS_WRITE: i32 = 1;
const SYS_OPEN: i32 = 2;
const SYS_CLOSE: i32 = 3;
const SYS_IOCTL: i32 = 16;

#[unsafe(no_mangle)]
pub extern "C" fn syscall(
    pid: i32,
    nr: i32,
    a0: i32,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i32 {
    unsafe {
        let kernel = &raw mut KERNEL;

        match nr {
            SYS_WRITE => {
                let fd = a0;
                let ptr = a1 as *const u8;
                let len = a2 as usize;

                let proc = match (*kernel).get_or_create_process(pid) {
                    Some(p) => p,
                    None => return ERR,
                };

                let fd_desc = match proc.fd_table.get_mut(fd) {
                    Some(desc) => desc as *mut _,
                    None => return ERR,
                };

                let mut buf = vec![0u8; len];
                cp_from_bin(pid, ptr, buf.as_mut_ptr(), len);
                (*kernel).vfs.write(&mut *fd_desc, buf.as_slice())
            }
            SYS_READ => {
                let fd = a0;
                let ptr = a1 as *mut u8;
                let len = a2 as usize;

                let proc = match (*kernel).get_or_create_process(pid) {
                    Some(p) => p,
                    None => return ERR,
                };

                let fd_desc = match proc.fd_table.get_mut(fd) {
                    Some(desc) => desc as *mut _,
                    None => return ERR,
                };

                let mut buf = vec![0u8; len];
                let res = (*kernel).vfs.write(&mut *fd_desc, buf.as_mut_slice());
                cp_to_bin(pid, ptr, buf.as_ptr(), len);
                res
            }

            SYS_OPEN => {
                let path_ptr = a0 as *const u8;
                let path_len = a1 as usize;
                let flags = a2 as u32;

                let mut buf = vec![0u8; path_len];
                cp_from_bin(pid, path_ptr, buf.as_mut_ptr(), path_len);
                let path_str = match core::str::from_utf8(buf.as_slice()) {
                    Ok(s) => s,
                    Err(_) => return ERR,
                };

                let proc = match (*kernel).get_or_create_process(pid) {
                    Some(p) => p,
                    None => return ERR,
                };

                let fd_table_ptr = &mut proc.fd_table as *mut _;

                (*kernel).vfs.open(path_str, flags, &mut *fd_table_ptr)
            }

            SYS_CLOSE => {
                let fd = a0;

                let proc = match (*kernel).get_or_create_process(pid) {
                    Some(p) => p,
                    None => return ERR,
                };

                proc.fd_table.close(fd)
            }

            SYS_IOCTL => {
                let fd = a0;
                let cmd = a1 as u32;
                let arg = a2 as usize;

                let proc = match (*kernel).get_or_create_process(pid) {
                    Some(p) => p,
                    None => return ERR,
                };

                let fd_desc = match proc.fd_table.get(fd) {
                    Some(desc) => desc as *const _,
                    None => return ERR,
                };

                (*kernel).vfs.ioctl(&*fd_desc, cmd, arg)
            }
            _ => i32::MAX,
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static GLOBAL_ALLOC: KernelAlloc = KernelAlloc;
