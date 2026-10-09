use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use regex::Regex;

fn to_rust_type(ty: &str) -> Result<Option<&str>, String> {
    let ty = ty.trim();
    Ok(Some(match ty {
        "void" => return Ok(None),
        "uint32_t" => "u32",
        "uint8_t" => "u8",
        "int" | "int32_t" => "i32",
        "double" => "f64",
        "bool" => "bool",
        "char *" => "*const c_char",
        "brain *" => "*mut RawBrain",
        "motor *" => "*mut RawMotor",
        "inertial *" => "*mut RawInertial",
        "distance *" => "*mut RawDistance",
        "smartdrive *" => "*mut RawSmartdrive",
        "DriveDirection" | "TurnDirection" => ty,
        _ => return Err(format!("Unknown type: {}", ty)),
    }))
}

fn main() {
    let _ = dotenvy::dotenv();

    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=VEX_SDK_PATH");
    println!("cargo:rerun-if-env-changed=VEX_TOOLS_PATH");

    let sdk = std::env::var("VEX_SDK_PATH").expect("missing VEX_SDK_PATH");

    println!("cargo:rustc-link-arg=-T{sdk}/lscript.ld");
    println!("cargo:rustc-link-arg=--no-warn-rwx-segments");
    println!("cargo:rustc-link-search=native={sdk}");
    println!("cargo:rustc-link-search=native={sdk}/gcc/libs");
    for lib in ["exprt", "c_nano", "stdc++_nano", "m", "gcc"] {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    println!("cargo:rerun-if-changed=shim/shim.cpp");

    cc::Build::new()
        .cpp(true)
        .cpp_link_stdlib(None)
        .std("gnu++17")
        .flags([
            "-mcpu=cortex-m7",
            "-fshort-enums",
            "-fno-rtti",
            "-fno-exceptions",
            "-fno-threadsafe-statics",
            "-Werror=return-type",
            "-U__INT32_TYPE__",
            "-U__UINT32_TYPE__",
        ])
        .define("__INT32_TYPE__", "long")
        .define("__UINT32_TYPE__", "unsigned long")
        .flag(format!("-isystem{sdk}/include"))
        .flag(format!("-isystem{sdk}/gcc/include"))
        .flag(format!("-isystem{sdk}/gcc/include/c++/7.3.1"))
        .flag(format!("-isystem{sdk}/gcc/include/c++/7.3.1/arm-none-eabi"))
        .file("shim/shim.cpp")
        .compile("shim");

    let shim_src = fs::read_to_string("shim/shim.cpp").unwrap();
    let re = Regex::new(r"([a-z0-9_]+ \*?)(exp_[a-z_]+)\((.*?)\)").unwrap();
    let arg_re = Regex::new(r"([a-zA-Z0-9_]+ \*?)([a-z]+)").unwrap();
    let mut out = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("src/ffi.rs")
        .unwrap();
    write!(
        out,
        r#"use super::*;
use core::{{
    ffi::c_char,
    marker::{{PhantomData, PhantomPinned}},
        }};

macro_rules! opaque {{
    ($($name:ident),+ $(,)?) => {{
        $(
            #[repr(C)]
            pub struct $name {{
                _data: [u8; 0],
                _marker: PhantomData<(*mut u8, PhantomPinned)>,
            }}
        )+
    }};
}}

opaque!(RawBrain, RawMotor, RawInertial, RawDistance, RawSmartdrive);

unsafe extern "C" {{
"#
    )
    .unwrap();
    for captures in re.captures_iter(&shim_src) {
        write!(out, "    pub fn {}(", captures.get(2).unwrap().as_str()).unwrap();
        let args: Vec<_> = captures.get(3).unwrap().as_str().split(',').collect();
        for (i, arg) in args.iter().enumerate() {
            if arg.is_empty() {
                continue;
            }
            let captures = arg_re.captures(arg).unwrap();
            write!(
                out,
                "{}: {}",
                captures.get(2).unwrap().as_str(),
                to_rust_type(captures.get(1).unwrap().as_str())
                    .unwrap()
                    .expect("Unexpected void")
            )
            .unwrap();
            if i != args.len() - 1 {
                write!(out, ", ").unwrap();
            }
        }
        write!(out, ")").unwrap();
        if let Some(t) = to_rust_type(captures.get(1).unwrap().as_str()).unwrap() {
            write!(out, " -> {t}").unwrap();
        }
        write!(out, ";\n").unwrap();
    }
    write!(out, "}}\n").unwrap();
}
