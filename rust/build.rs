use std::env;

fn main() {
    println!("cargo:rustc-env=PROFILE={}", env::var("PROFILE").unwrap());
    println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
    println!(
        "cargo:rustc-env=OPT_LEVEL={}",
        env::var("OPT_LEVEL").unwrap()
    );
    println!(
        "cargo:rustc-env=CARGO_CFG_TARGET_FEATURE={}",
        env::var("CARGO_CFG_TARGET_FEATURE").unwrap()
    );
}
