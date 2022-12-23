use bindgen;
use pkg_config;
use std::env;
use std::path::PathBuf;

fn main() {
    let pmix = pkg_config::Config::new()
        .atleast_version("4.2.0")
        .probe("pmix")
        .expect("Could not find valid PMIx installation");
    let bind = bindgen::Builder::default()
        .header("src/bindings.h")
        // Invalid build on header change
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(
            pmix.include_paths
                .iter()
                .map(|path| format!("-I{}", path.as_os_str().to_str().unwrap())),
        )
        .allowlist_function("PMIx_.*")
        .generate()
        .expect("Failed to generate bindings");
    let mut path = PathBuf::from(env::var("OUT_DIR").unwrap());
    path.push("bindings.rs");
    bind.write_to_file(path)
        .expect("Failed to write PMIx bindings");
}
