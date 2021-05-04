extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let here_file = PathBuf::from(file!()).canonicalize().unwrap();
    let here = here_file.parent().unwrap();
    Command::new("cp")
        .current_dir(&here)
        .arg("-a")
        .arg((&here).join("libpfm4").to_str().unwrap())
        .arg((&out_path).to_str().unwrap())
        .status()
        .unwrap();
    let libpfm_dir = out_path.join("libpfm4");
    Command::new("make")
        .env("CFLAGS", "-fPIC")
        .current_dir(&libpfm_dir)
        .status()
        .unwrap();

    println!(
        "cargo:rustc-link-search=native={}",
        &libpfm_dir.join("lib").to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static=pfm");
    let include_dir = &libpfm_dir.join("include");

    let header = &include_dir
        .join("perfmon")
        .join("pfmlib_perf_event.h");
    println!("cargo:rerun-if-changed={}", (&header).to_str().unwrap());

    let bindings = bindgen::Builder::default()
        .allowlist_type("^(pfm|PFM).*")
        .allowlist_function("^(pfm|PFM).*")
        .allowlist_var("^(pfm|PFM).*")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .header((&header).to_str().unwrap())
        .clang_arg(format!("-I{}", (&include_dir).to_str().unwrap()))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
