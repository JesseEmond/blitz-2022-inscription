use std::{env, iter::FromIterator, path::PathBuf};

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

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

    let vendor_dlx = PathBuf::from_iter(&["vendor", "dlx"]);

    println!(
        "cargo:rerun-if-changed={}",
        vendor_dlx.join("dlx.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendor_dlx.join("dlx.c").display()
    );

    cc::Build::new()
        .file(vendor_dlx.join("dlx.c"))
        .include(&vendor_dlx)
        .flag_if_supported("-Wno-unused-parameter")
        .compile("libdlx.a");

    bindgen::builder()
        .header(vendor_dlx.join("dlx.h").to_string_lossy())
        .allowlist_function("dlx_.*")
        .allowlist_type("dlx_.*")
        .clang_arg(format!("-I{}", vendor_dlx.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("DLX bindings failure")
        .write_to_file(out_path.join("bindings_dlx.rs"))
        .expect("DLX bindings write failure");
}
