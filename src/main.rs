#![no_std]
#![no_main]

use rust_exp::Brain;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let brain = Brain::new().unwrap();
    brain.print_at(2, 30, "Hello EXP").unwrap();
}
