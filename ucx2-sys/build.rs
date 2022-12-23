use bindgen;
use pkg_config;
use cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let ucx = pkg_config::Config::new()
        .atleast_version("1.12")
        .probe("ucx")
        .expect("Could not find ucx library");

    // Set the proper link paths
    for link_path in ucx.link_paths {
        println!("cargo:rustc-link-search={}", link_path.as_os_str().to_str().unwrap());
    }
    println!("cargo:rerun-if-changed=src/ucx.h");
    println!("cargo:rerun-if-changed=src/ucx.c");

    // Generate and dump the bindings
    let bindings = bindgen::Builder::default()
        .header("src/ucx.h")
        // Just ucp for now
        // .allowlist_function("ucp_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .prepend_enum_name(false)
        .generate()
        .expect("Failed to generate bindings");
    let mut path = PathBuf::from(env::var("OUT_DIR").unwrap());
    path.push("bindings.rs");
    bindings
        .write_to_file(path)
        .expect("Failed to write UCX bindings");

    // Build and link the static wrapper code
    cc::Build::new()
        .file("src/ucx.c")
        .include("src")
        .includes(ucx.include_paths)
        .compile("ucx");
}
