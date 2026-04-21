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

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("build.rs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        submodule_replay_version_header.display()
    );

    if let Some(version) = workspace_replay_version(&submodule_replay_version_header) {
        println!("cargo:rustc-env=RENDERDOG_REPLAY_WORKSPACE_VERSION={version}");
    }

    if env::var_os("CARGO_FEATURE_CXX_REPLAY").is_none() {
        return;
    }

    let replay_include_dir = manifest_dir.join("include");
    let renderdoc_replay_api_dir = workspace_root
        .join("third-party")
        .join("renderdoc")
        .join("renderdoc")
        .join("api")
        .join("replay");

    if !renderdoc_replay_api_dir.exists() {
        panic!(
            "RenderDoc headers not found at {:?}. Did you init the `third-party/renderdoc` submodule?",
            renderdoc_replay_api_dir
        );
    }

    let mut build = cxx_build::bridge("src/ffi.rs");
    build.file("src/replay.cc");
    build.include(replay_include_dir);
    build.include(renderdoc_replay_api_dir);

    build.flag_if_supported("-std=c++14");

    if cfg!(windows) {
        build.define("RENDERDOC_PLATFORM_WIN32", None);
        build.flag_if_supported("/EHsc");
    } else if cfg!(all(unix, target_os = "linux")) {
        build.define("RENDERDOC_PLATFORM_LINUX", None);
        println!("cargo:rustc-link-lib=dl");
    } else if cfg!(target_os = "macos") {
        build.define("RENDERDOC_PLATFORM_APPLE", None);
        // RenderDoc's upstream replay headers currently trigger a few Clang-only warnings on macOS.
        // Keep our shim warnings visible while suppressing the known third-party noise introduced by
        // newer RenderDoc versions.
        build.flag_if_supported("-Wno-unused-parameter");
        build.flag_if_supported("-Wno-deprecated-literal-operator");
        build.flag_if_supported("-Wno-reorder-ctor");
    }

    build.compile("renderdog_replay");

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/replay.cc");
    println!("cargo:rerun-if-changed=include/replay.h");
    println!("cargo:rerun-if-changed=include/renderdoc_runtime_api.h");
}

fn workspace_replay_version(submodule_header: &Path) -> Option<String> {
    if submodule_header.is_file() {
        let content = fs::read_to_string(submodule_header).ok()?;
        return parse_replay_version_header(&content);
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
