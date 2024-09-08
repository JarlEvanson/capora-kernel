//! Build script for `kernel`.

fn main() {
    println!("cargo::rustc-link-arg=-Tkernel/linker_script.ld");
}
