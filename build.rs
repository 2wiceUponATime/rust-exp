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
}
