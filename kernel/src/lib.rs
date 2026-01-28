#![no_std]
#![no_main]

pub type Fd = i32;
pub type Pid = i32;

pub type ResultCode = i32;

pub const OK: ResultCode = 0;
pub const ERR: ResultCode = -1;

#[link(wasm_import_module = "sys")]
unsafe extern "C" {
    pub fn os_write(fd: Fd, ptr: *const u8, len: usize) -> ResultCode;
    pub fn os_read(fd: Fd, ptr: *mut u8, len: usize) -> ResultCode;
    pub fn os_open(path_ptr: *const u8, path_len: usize, flags: u32) -> Fd;

    pub fn os_ipc_send(pid: Pid, ptr: *const u8, len: usize) -> ResultCode;
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {}

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
        match nr {
            1 => os_write(a0, a1 as *const u8, a2 as usize),
            _ => i32::MAX,
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
