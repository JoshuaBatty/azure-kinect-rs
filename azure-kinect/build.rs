extern crate bindgen;

use std::env;
use std::env::consts;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(r#"-IC:\Program Files\Azure Kinect SDK v1.4.1\sdk\include"#)
        .clang_arg("-v")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_function("k4a.*")
        .whitelist_type("_?[kK]4[aA].*")
        .whitelist_var("[kK]4[aA].*")
        .rustified_enum("[kK]4[aA].*")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
