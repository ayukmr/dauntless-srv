use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=www/src");

    let status = Command::new("yarn")
        .arg("build")
        .current_dir("www")
        .status()
        .expect("failed to run web build");

    if !status.success() {
        panic!("web build failed");
    }
}
