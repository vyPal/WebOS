#![no_std]
#![no_main]

use os_sdk::*;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let null_fd = open("/dev/null", O_WRONLY);
    if null_fd >= 0 {
        write(null_fd, b"Test write to /dev/null\n");
        close(null_fd);
    }

    let console_fd = open("/dev/serial", O_WRONLY);
    if console_fd >= 0 {
        write(console_fd, b"Test write to /dev/serial\n");
        close(console_fd);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
