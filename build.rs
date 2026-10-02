fn main() {
    let _ = dotenvy::dotenv();

    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=VEX_SDK_PATH");
    println!("cargo:rerun-if-env-changed=VEX_TOOLS_PATH");

    let sdk = std::env::var("VEX_SDK_PATH").expect("missing VEX_SDK_PATH");
    let tools = std::env::var("VEX_TOOLS_PATH").expect("missing VEX_TOOLS_PATH");

    println!("cargo:rustc-link-arg=-T{sdk}/lscript.ld");
    println!("cargo:rustc-link-search=native={sdk}");
    println!("cargo:rustc-link-search=native={sdk}/gcc/libs");
    for lib in ["exprt", "c_nano", "stdc++_nano", "m", "gcc"] {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    cc::Build::new()
        .cpp(true)
        .cpp_link_stdlib(None)
        .compiler(format!("{tools}/clang/bin/clang"))
        .std("gnu++17")
        .flags([
            "--target=thumbv7-none-eabi",
            "-fshort-enums",
            "-Wno-unknown-attributes",
            "-Wno-unused-parameter",
            "-U__INT32_TYPE__",
            "-U__UINT32_TYPE__",
            "-D__INT32_TYPE__=long",
            "-D__UINT32_TYPE__=unsigned long",
            "-mthumb",
            "-mcpu=cortex-m7",
            "-mfpu=fpv5-d16",
            "-mfloat-abi=hard",
            "-Werror=return-type",
            "-fno-rtti",
            "-fno-threadsafe-statics",
            "-fno-exceptions",
            "-std=gnu++17",
        ])
        .include(format!("{sdk}/include"))
        .include(format!("{sdk}/clang/8.0.0/include"))
        .include(format!("{sdk}/gcc/include"))
        .include(format!("{sdk}/gcc/include/c++/7.3.1"))
        .include(format!("{sdk}/gcc/include/c++/7.3.1/arm-none-eabi"))
        .file("shim/shim.cpp")
        .compile("shim");
}
