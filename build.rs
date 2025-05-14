use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = Path::new(&manifest_dir).join("target");

    // Copy the built library to plugins directory
    let lib_name = env::var("CARGO_PKG_NAME").unwrap();
    let src = target_dir
        .join("release")
        .join(format!("lib{}.so", lib_name));
    let dst = Path::new(&manifest_dir).join(format!("lib{}.so", lib_name));

    println!("cargo:warning=Source path: {:?}", src);
    println!("cargo:warning=Destination path: {:?}", dst);

    // Only try to copy if the source file exists
    if src.exists() {
        println!("cargo:warning=Source file exists, attempting to copy");
        if let Err(e) = fs::copy(&src, &dst) {
            println!("cargo:warning=Failed to copy library: {}", e);
        } else {
            println!("cargo:warning=Successfully copied library to {:?}", dst);
        }
    } else {
        println!("cargo:warning=Source file does not exist at {:?}", src);
    }
}
