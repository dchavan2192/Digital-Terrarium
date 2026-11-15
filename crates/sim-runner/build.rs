use std::process::Command;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let git = |args: &[&str]| -> Option<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
    };
    let compiler = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=TERRARIUM_COMPILER={compiler}");
    let revision = git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = git(&["status", "--porcelain", "--untracked-files=normal"])
        .map(|s| (!s.is_empty()).to_string())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=TERRARIUM_SOURCE_REVISION={revision}");
    println!("cargo:rustc-env=TERRARIUM_SOURCE_DIRTY={dirty}");
    println!(
        "cargo:rustc-env=TERRARIUM_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    println!(
        "cargo:rustc-env=TERRARIUM_BUILD_PROFILE={}",
        std::env::var("PROFILE").unwrap()
    );
    // Track the compiled workspace and Git identity, including dependency changes.
    for input in ["Cargo.toml", "Cargo.lock", "crates", "configs"] {
        println!("cargo:rerun-if-changed={}", root.join(input).display());
    }
    for input in ["HEAD", "index", "refs"] {
        if let Some(path) = git(&["rev-parse", "--path-format=absolute", "--git-path", input]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}
