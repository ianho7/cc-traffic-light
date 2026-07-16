use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    if cfg!(target_os = "windows") {
        let manifest_path =
            std::fs::canonicalize("app.manifest").expect("failed to canonicalize app.manifest");
        let icon_path = std::fs::canonicalize("../taskbar-settings-tauri/src-tauri/icons/icon.ico")
            .expect("failed to canonicalize application icon");
        let output_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
        let resource_path = output_dir.join("cc_traffic_light_icon.res");

        println!("cargo:rerun-if-changed=app.manifest");
        println!("cargo:rerun-if-changed={}", icon_path.display());
        println!("cargo:rerun-if-changed=app.rc");

        compile_icon_resource(&resource_path);

        println!("cargo:rustc-link-arg-bin=taskbar-widget=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=taskbar-widget=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
        println!(
            "cargo:rustc-link-arg-bin=taskbar-widget={}",
            resource_path.display()
        );
        println!(
            "cargo:rustc-link-arg-bin=taskbar_widget_hook={}",
            resource_path.display()
        );
    }
}

fn compile_icon_resource(resource_path: &Path) {
    let compiler = find_resource_compiler().expect(
        "Windows resource compiler was not found; install the Windows SDK or set the RC environment variable",
    );
    let resource_script = PathBuf::from("app.rc");
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg(format!("/fo{}", resource_path.display()))
        .arg(resource_script)
        .status()
        .expect("failed to start Windows resource compiler");

    assert!(status.success(), "Windows resource compiler failed");
}

fn find_resource_compiler() -> Option<PathBuf> {
    if let Some(path) = env::var_os("RC") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let sdk_root = env::var_os("WindowsSdkDir")
        .map(PathBuf::from)
        .or_else(|| env::var_os("ProgramFiles(x86)").map(|root| PathBuf::from(root).join("Windows Kits\\10")))?;
    let bin_dir = sdk_root.join("bin");
    let mut versions = fs::read_dir(bin_dir).ok()?.filter_map(Result::ok).collect::<Vec<_>>();
    versions.sort_by_key(|entry| entry.file_name());

    versions.into_iter().rev().find_map(|entry| {
        let compiler = entry.path().join("x64").join("rc.exe");
        compiler.is_file().then_some(compiler)
    })
}
