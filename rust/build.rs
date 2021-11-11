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

    let flags = env::var("RUSTFLAGS").unwrap_or_default();
    println!(
        "cargo:rustc-env=PGO_USE={}",
        if flags.contains("profile-use=") { "yes" } else { "no" },
    );

    let dlx_mod = PathBuf::from_iter(&["src", "dlx"]);

    let dlx_h = dlx_mod.join("dlx.h");
    let dlx_c = dlx_mod.join("dlx.c");

    println!("cargo:rerun-if-changed={}", dlx_h.display());
    println!("cargo:rerun-if-changed={}", dlx_c.display());

    cc::Build::new()
        .file(&dlx_c)
        .flag_if_supported("-Wno-unused-parameter")
        .compile("libdlx.a");

    bindgen::builder()
        .header(dlx_h.to_string_lossy())
        .allowlist_function("dlx_.*")
        .allowlist_type("dlx_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("DLX bindings failure")
        .write_to_file(out_path.join("bindings_dlx.rs"))
        .expect("DLX bindings write failure");
}
