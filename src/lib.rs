#![no_std]

#[cfg(rust_analyzer)]
#[path = "../bindings/mod.rs"]
mod bindings_src;

extern crate alloc;
mod allocator;

use core::panic::PanicInfo;

pub use generated::*;
pub mod units;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    let _ = io::flush();
    request_exit();
    loop {
        thread::sleep(1000);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
        let _ = $crate::io::flush();
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
        let _ = $crate::io::flush();
    };
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
