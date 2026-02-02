#![no_std]

pub type Fd = i32;
pub type Pid = i32;

pub type ResultCode = i32;

pub const OK: ResultCode = 0;
pub const ERR: ResultCode = -1;

pub const O_RDONLY: u32 = 0x0000;
pub const O_WRONLY: u32 = 0x0001;
pub const O_RDWR: u32 = 0x0002;
pub const O_CREAT: u32 = 0x0040;
pub const O_TRUNC: u32 = 0x0200;

const SYS_READ: i32 = 0;
const SYS_WRITE: i32 = 1;
const SYS_OPEN: i32 = 2;
const SYS_CLOSE: i32 = 3;
const SYS_IOCTL: i32 = 16;

#[link(wasm_import_module = "kernel")]
unsafe extern "C" {
    pub fn syscall(nr: i32, a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) -> i32;
}

pub fn write(fd: Fd, buf: &[u8]) -> ResultCode {
    unsafe {
        syscall(
            SYS_WRITE,
            fd,
            buf.as_ptr() as i32,
            buf.len() as i32,
            0,
            0,
            0,
        )
    }
}

pub fn read(fd: Fd, buf: &mut [u8]) -> ResultCode {
    unsafe {
        syscall(
            SYS_READ,
            fd,
            buf.as_mut_ptr() as i32,
            buf.len() as i32,
            0,
            0,
            0,
        )
    }
}

pub fn open(path: &str, flags: u32) -> Fd {
    unsafe {
        let path = path.as_bytes();
        syscall(
            SYS_OPEN,
            path.as_ptr() as i32,
            path.len() as i32,
            flags as i32,
            0,
            0,
            0,
        )
    }
}

pub fn close(fd: Fd) -> ResultCode {
    unsafe { syscall(SYS_CLOSE, fd, 0, 0, 0, 0, 0) }
}

pub fn ioctl(fd: Fd, cmd: u32, arg: usize) -> ResultCode {
    unsafe { syscall(SYS_IOCTL, fd, cmd as i32, arg as i32, 0, 0, 0) }
}
