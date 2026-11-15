//! Non-cryptographic build identity for checkpoint compatibility. This is a
//! reproducibility guard, not authentication of untrusted checkpoint files.
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read core source directory") {
        let path = entry.expect("read source entry").path();
        if path.is_dir() {
            collect(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut files = vec![root.join("Cargo.toml"), root.join("build.rs")];
    collect(&root.join("src"), &mut files);
    files.push(root.join("../../Cargo.toml"));
    files.push(root.join("../../Cargo.lock"));
    files.sort();
    let mut fingerprint = 0xcbf29ce484222325_u64;
    let mut feed = |bytes: &[u8]| {
        for b in bytes {
            fingerprint ^= u64::from(*b);
            fingerprint = fingerprint.wrapping_mul(0x100000001b3);
        }
    };
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        feed(
            path.strip_prefix(&root)
                .unwrap_or(&path)
                .to_string_lossy()
                .as_bytes(),
        );
        feed(&fs::read(&path).expect("read build identity input"));
    }
    let compiler = Command::new(env::var_os("RUSTC").unwrap())
        .arg("-Vv")
        .output()
        .expect("rustc identity");
    feed(&compiler.stdout);
    let mut settings: Vec<_> = env::vars()
        .filter(|(k, _)| {
            k.starts_with("CARGO_FEATURE_")
                || [
                    "TARGET",
                    "PROFILE",
                    "OPT_LEVEL",
                    "DEBUG",
                    "CARGO_ENCODED_RUSTFLAGS",
                ]
                .contains(&k.as_str())
        })
        .collect();
    settings.sort();
    for (key, value) in settings {
        feed(key.as_bytes());
        feed(value.as_bytes());
    }
    println!("cargo:rustc-env=TERRARIUM_CORE_BUILD={fingerprint:016x}");
    println!(
        "cargo:rustc-env=TERRARIUM_CORE_TARGET={}",
        env::var("TARGET").unwrap()
    );
}
