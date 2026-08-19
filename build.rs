#[cfg(windows)]
fn main() {
    use std::path::PathBuf;

    // Dynamically locate Windows Kits rc.exe if not in PATH
    let win_kit_dirs = [
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64",
    ];

    let mut toolkit_path = None;
    for dir in &win_kit_dirs {
        let p = PathBuf::from(dir);
        if p.join("rc.exe").exists() {
            toolkit_path = Some(dir.to_string());
            break;
        }
    }

    let mut res = winres::WindowsResource::new();
    if let Some(ref path) = toolkit_path {
        res.set_toolkit_path(path);
    }
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "Titan Browser");
    res.set("FileDescription", "Titan Browser - Fast Web Browser built in Rust");
    res.set("LegalCopyright", "Copyright (C) 2026 Titan Browser Team");

    res.compile().expect("Failed to compile Windows resource with icon");

    let _ = std::process::Command::new("cmd")
        .args(["/c", "npm run build"])
        .status();

    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=ui/src/app.ts");
    println!("cargo:rerun-if-changed=ui/src/types.d.ts");
    println!("cargo:rerun-if-changed=ui/index.html");
    println!("cargo:rerun-if-changed=ui/style.css");
}

#[cfg(not(windows))]
fn main() {}
