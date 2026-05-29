// crates/web/build.rs

fn main() {
    // Askama template derleme tetikleyici build script
    println!("cargo:rerun-if-changed=templates");
}
