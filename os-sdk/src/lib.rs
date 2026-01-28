#![no_std]

pub type Fd = i32;
pub type Pid = i32;

pub type ResultCode = i32;

pub const OK: ResultCode = 0;
pub const ERR: ResultCode = -1;

#[link(wasm_import_module = "kernel")]
unsafe extern "C" {
    pub fn syscall(nr: i32, a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) -> i32;
}

pub fn write(fd: Fd, buf: &[u8]) -> ResultCode {
    unsafe { syscall(1, fd, buf.as_ptr() as i32, buf.len() as i32, 0, 0, 0) }
}
