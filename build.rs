fn main() {
    println!("cargo:rerun-if-changed=www/src");

    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("yarn")
            .arg("build")
            .current_dir("www")
            .status()
            .expect("failed to run web build");

        if !status.success() {
            panic!("web build failed");
        }
    }
}
