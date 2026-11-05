fn main() {
    let _ = dotenvy::dotenv();

    if let Err(e) = rust_exp_build::Builder::new()
        .cpp(include_str!("shim/prelude.cpp"))
        .build()
    {
        panic!("{e}");
    }

    return;
}
