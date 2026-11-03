use std::{fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;

const SHIM_FILE: &str = "shim/shim.cpp";

fn to_rust_type(ty: &str) -> Result<TokenStream, String> {
    Ok(match ty {
        "void" => quote! { () },
        "uint32_t" => quote! { u32 },
        "uint8_t" => quote! { u8 },
        "int" | "int32_t" => quote! { i32 },
        "double" => quote! { f64 },
        "bool" => quote! { bool },
        "size_t" => quote! { usize },
        "char*" => quote! { *const c_char },
        "drivetrain*" => quote! { *mut RawDrivetrain },
        "brain*" => quote! { *mut RawBrain },
        "motor*" => quote! { *mut RawMotor },
        "motor**" => quote! { *const *mut RawMotor },
        "motor_group*" => quote! { *mut RawMotorGroup },
        "inertial*" => quote! { *mut RawInertial },
        "distance*" => quote! { *mut RawDistance },
        "smartdrive*" => quote! { *mut RawSmartdrive },
        "Direction" => quote! { Direction },
        "TurnDirection" => quote! { TurnDirection },
        "BrakeType" => quote! { BrakeType },
        "VelocityUnits" => quote! { VelocityUnits },
        _ => return Err(format!("Unknown type: {}", ty)),
    })
}

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|e| panic!("failed to read env var {key}: {e}"))
}

fn write_compile_commands(build: &cc::Build, file: &str) {
    let root = env("CARGO_MANIFEST_DIR");
    let tool = build.get_compiler();

    let mut arguments = vec![tool.path().to_string_lossy().into_owned()];
    arguments.extend(
        tool.args()
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned()),
    );
    arguments.extend(["--target=arm-none-eabi", "-nostdlibinc", "-c", file].map(String::from));

    let db = serde_json::json!([{
        "directory": root,
        "file": file,
        "arguments": arguments,
    }]);
    fs::write(
        format!("{root}/compile_commands.json"),
        serde_json::to_string_pretty(&db).unwrap(),
    )
    .unwrap();
}

fn main() {
    let _ = dotenvy::dotenv();

    println!("cargo::rerun-if-changed=.env");
    println!("cargo::rerun-if-env-changed=VEX_SDK_PATH");

    let sdk = env("VEX_SDK_PATH");
    let out_dir = PathBuf::from(env("OUT_DIR"));

    println!("cargo::rustc-link-arg=-T{sdk}/lscript.ld");
    println!("cargo::rustc-link-arg=--no-warn-rwx-segments");
    println!("cargo::rustc-link-search=native={sdk}");
    println!("cargo::rustc-link-search=native={sdk}/gcc/libs");
    for lib in ["exprt", "c_nano", "stdc++_nano", "m", "gcc"] {
        println!("cargo::rustc-link-lib=static={lib}");
    }

    println!("cargo::rerun-if-changed={SHIM_FILE}");

    let mut build = cc::Build::new();
    build
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
        .flags(
            [
                "include",
                "gcc/include",
                "gcc/include/c++/7.3.1",
                "gcc/include/c++/7.3.1/arm-none-eabi",
            ]
            .map(|dir| format!("-isystem{sdk}/{dir}")),
        )
        .define("__INT32_TYPE__", "long")
        .define("__UINT32_TYPE__", "unsigned long")
        .file(SHIM_FILE);
    write_compile_commands(&build, SHIM_FILE);

    let shim_src = fs::read_to_string(SHIM_FILE).unwrap();
    let re = Regex::new(r"([a-zA-Z0-9_]+\**) (exp_[a-z_]+)\(([\s\S]*?)\)").unwrap();
    let arg_re = Regex::new(r"([a-zA-Z0-9_]+\**) ([a-z_]+)").unwrap();
    let decls = re.captures_iter(&shim_src).map(|captures| {
        let arg_strings: Vec<&str> = captures.get(3).unwrap().as_str().split(',').collect();
        let args = arg_strings.iter().filter_map(|arg| {
            let captures = arg_re.captures(arg)?;
            let arg = format_ident!("{}", &captures[2]);
            let ty = to_rust_type(&captures[1]).unwrap();
            Some(quote! { #arg: #ty })
        });
        let name = format_ident!("{}", &captures[2]);
        let ret = to_rust_type(&captures[1]).unwrap();
        quote! {
            pub fn #name(#(#args),*) -> #ret;
        }
    });

    let tokens = quote! {
        use super::*;
        use core::{
            ffi::c_char,
            marker::{PhantomData, PhantomPinned},
        };

        macro_rules! opaque {
            ($($name:ident),+ $(,)?) => {
                $(
                    #[repr(C)]
                    pub struct $name {
                        _data: [u8; 0],
                        _marker: PhantomData<(*mut u8, PhantomPinned)>,
                    }
                )+
            };
        }

        opaque!(
            RawDrivetrain,
            RawSmartdrive,
            RawMotor,
            RawMotorGroup,
            RawBrain,
            RawInertial,
            RawDistance
        );

        unsafe extern "C" {
            #(#decls)*
        }
    };

    let file = syn::parse2(tokens).unwrap();
    std::fs::write(out_dir.join("ffi.rs"), prettyplease::unparse(&file)).unwrap();

    build.compile("shim");
}
