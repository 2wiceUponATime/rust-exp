#![no_std]
#![no_main]

use rust_exp::println;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    println!("Hello, World!");
}
