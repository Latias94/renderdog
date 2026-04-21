use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("..").join("..");
    let submodule_replay_version_header = workspace_root
        .join("third-party")
        .join("renderdoc")
        .join("renderdoc")
        .join("api")
        .join("replay")
        .join("version.h");
    let vendored_replay_version = manifest_dir
        .join("vendor")
        .join("renderdoc_replay_version.txt");

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("build.rs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        submodule_replay_version_header.display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendored_replay_version.display()
    );

    if let Some(version) =
        workspace_replay_version(&submodule_replay_version_header, &vendored_replay_version)
    {
        println!("cargo:rustc-env=RENDERDOG_AUTOMATION_WORKSPACE_REPLAY_VERSION={version}");
    }
}

fn workspace_replay_version(submodule_header: &Path, vendored_version: &Path) -> Option<String> {
    if submodule_header.is_file() {
        let content = fs::read_to_string(submodule_header).ok()?;
        return parse_replay_version_header(&content);
    }

    if vendored_version.is_file() {
        let version = fs::read_to_string(vendored_version).ok()?;
        let version = version.trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }

    None
}

fn parse_replay_version_header(content: &str) -> Option<String> {
    let mut major: Option<String> = None;
    let mut minor: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("#define RENDERDOC_VERSION_MAJOR") {
            major = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("#define RENDERDOC_VERSION_MINOR") {
            minor = Some(value.trim().to_string());
        }
    }

    match (major, minor) {
        (Some(major), Some(minor)) => Some(format!("{major}.{minor}")),
        _ => None,
    }
}
