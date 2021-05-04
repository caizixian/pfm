extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

const LIBPFM_VERSION: &'static str = "4.11.0";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let libpfm_filename = format!("libpfm-{}.tar.gz", LIBPFM_VERSION);
    let download_url = format!(
        "https://sourceforge.net/projects/perfmon2/files/libpfm4/{}/download",
        libpfm_filename
    );
    Command::new("wget")
        .current_dir(&out_path)
        .arg("-O")
        .arg(&libpfm_filename)
        .arg(download_url)
        .status()
        .unwrap();

    Command::new("tar")
        .current_dir(&out_path)
        .arg("xf")
        .arg(&libpfm_filename)
        .status()
        .unwrap();
    let libpfm_dir = out_path.join(format!("libpfm-{}", LIBPFM_VERSION));
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

    let header = &libpfm_dir
        .join("include")
        .join("perfmon")
        .join("pfmlib_perf_event.h");
    println!("cargo:rerun-if-changed={}", (&header).to_str().unwrap());

    let bindings = bindgen::Builder::default()
        .allowlist_type("^(pfm|PFM).*")
        .allowlist_function("^(pfm|PFM).*")
        .allowlist_var("^(pfm|PFM).*")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .header((&header).to_str().unwrap())
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
