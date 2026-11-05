#![no_std]

extern crate alloc;
mod allocator;

use core::{
    marker::{PhantomData, PhantomPinned},
    panic::PanicInfo,
};

pub use generated::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    let _ = flush();
    request_exit();
    loop {
        sleep(1000);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
        let _ = $crate::flush();
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
        let _ = $crate::flush();
    };
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
