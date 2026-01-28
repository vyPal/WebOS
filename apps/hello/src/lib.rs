#![no_std]
#![no_main]

use os_sdk::*;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    write(1, b"hello from wasm app\n");
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
