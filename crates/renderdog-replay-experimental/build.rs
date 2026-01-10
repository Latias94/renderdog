#[cfg(feature = "cxx-replay")]
fn main() {
    use std::{env, path::PathBuf};

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let replay_include_dir = manifest_dir.join("include");
    let renderdoc_replay_api_dir = manifest_dir
        .join("..")
        .join("..")
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
    }

    build.compile("renderdog_replay_experimental");

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/replay.cc");
    println!("cargo:rerun-if-changed=include/replay.h");
}

#[cfg(not(feature = "cxx-replay"))]
fn main() {}

