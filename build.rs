// build.rs

fn main() {
    // Sqlx offline / template derleme tetikleyici build script
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=templates");
}
