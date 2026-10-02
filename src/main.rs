#![no_std]
#![no_main]

use rust_exp::Brain;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let mut brain = Brain::new();
    brain.print_at(2, 30, "Hello EXP");
}
