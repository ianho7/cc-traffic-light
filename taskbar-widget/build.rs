fn main() {
    if cfg!(target_os = "windows") {
        let manifest_path =
            std::fs::canonicalize("app.manifest").expect("failed to canonicalize app.manifest");
        println!("cargo:rerun-if-changed=app.manifest");
        println!("cargo:rustc-link-arg-bin=taskbar-widget=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=taskbar-widget=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
    }
}
