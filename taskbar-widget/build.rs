fn main() {
    println!("cargo:rerun-if-changed=ui/settings.slint");
    slint_build::compile("ui/settings.slint").expect("failed to compile Slint settings UI");

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
