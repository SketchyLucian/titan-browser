use std::process::Command;

#[cfg(windows)]
fn compile_windows_resources() {
    use std::path::PathBuf;

    let win_kit_dirs = [
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64",
    ];

    let toolkit_path = win_kit_dirs
        .iter()
        .find(|dir| PathBuf::from(dir).join("rc.exe").exists());

    let mut resources = winres::WindowsResource::new();
    if let Some(path) = toolkit_path {
        resources.set_toolkit_path(path);
    }
    resources.set_icon("assets/icon.ico");
    resources.set("ProductName", "Titan Browser");
    resources.set(
        "FileDescription",
        "Titan Browser - Fast Web Browser built in Rust",
    );
    resources.set("LegalCopyright", "Copyright (C) 2026 Titan Browser Team");
    resources
        .compile()
        .expect("Failed to compile Windows resource with icon");
}

fn compile_typescript() {
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(npm)
        .args(["run", "build"])
        .status()
        .expect("Failed to start the TypeScript build. Run `npm install` first.");

    assert!(status.success(), "TypeScript build failed");
}

fn main() {
    #[cfg(windows)]
    compile_windows_resources();

    compile_typescript();

    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=tsconfig.json");
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/index.html");
    println!("cargo:rerun-if-changed=ui/newtab.html");
    println!("cargo:rerun-if-changed=ui/settings.html");
    println!("cargo:rerun-if-changed=ui/themes.html");
    println!("cargo:rerun-if-changed=ui/style.css");
    println!("cargo:rerun-if-changed=web-scripts/src");
    println!("cargo:rerun-if-changed=web-scripts/tsconfig.json");
}
