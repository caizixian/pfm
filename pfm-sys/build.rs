extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("OUT_DIR: {:?}", out_path);
    let here = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("CARGO_MANIFEST_DIR: {:?}", here);

    let status = Command::new("cp")
        .current_dir(&here)
        .arg("-a")
        .arg((&here).join("libpfm4").to_str().unwrap())
        .arg((&out_path).to_str().unwrap())
        .status()
        .unwrap();
    if !status.success() {
        panic!("cp exited with status {}", status);
    }
    
    let libpfm_dir = out_path.join("libpfm4");
    // When invoking cargo build within a currently running GNU make
    // Cargo uses the jobserver protocol
    // https://doc.rust-lang.org/cargo/reference/build-scripts.html#jobserver
    // And any child make will receive something like "--jobserver-fds=3,4" in
    // the MAKEFLAGS environment variable
    // However, the jobserver protocol doesn't specify whether the pipe is
    // non-blocking or not.
    // If the pipe is set to be non-blocking,
    // old versions (before 4.3)
    // https://bob-build-tool.readthedocs.io/en/latest/manual/configuration.html?highlight=jobserver#jobserver
    // of GNU make will give
    // the following error
    // *** read jobs pipe: Resource temporarily unavailable.  Stop.
    // Also see
    // https://lists.gnu.org/archive/html/bug-make/2019-08/msg00018.html
    let status = Command::new("make")
        .env("CFLAGS", "-fPIC")
        .env_remove("MAKEFLAGS")
        .current_dir(&libpfm_dir)
        .status()
        .unwrap();
    if !status.success() {
        panic!("make exited with status {}", status);
    }

    println!(
        "cargo:rustc-link-search=native={}",
        &libpfm_dir.join("lib").to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static=pfm");
    let include_dir = &libpfm_dir.join("include");

    let header = &include_dir.join("perfmon").join("pfmlib_perf_event.h");
    println!("cargo:rerun-if-changed={}", (&header).to_str().unwrap());

    let bindings = bindgen::Builder::default()
        .allowlist_type("^(pfm|PFM|perf|PERF).*")
        .allowlist_function("^(pfm|PFM|perf|PERF).*")
        .allowlist_var("^(pfm|PFM|perf|PERF).*")
        .rustified_enum("^pfm_.*_t$")
        .rustified_enum("^perf_.*$")
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
