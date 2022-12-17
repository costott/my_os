#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

mod vga_buffer;

use core::panic::PanicInfo;

/// called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! { // ingore this error
    println!("{}", info);
    loop {}
}

/// entry point, as the linker looks for a function names _start by default
#[no_mangle] // ensure the rust compiler really makes a function named `_start`
pub extern "C" fn _start() -> ! { // use the C calling convention
    println!("Hello World{}", "!");
    panic!("Test panic (this is intentional)");

    loop {}
}