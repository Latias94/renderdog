use std::{env, path::PathBuf};

fn main() {
    if env::var_os("CARGO_FEATURE_CXX_REPLAY").is_none() {
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
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

        let lib_dir = find_renderdoc_lib_dir();
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=dylib=renderdoc");
    } else if cfg!(all(unix, target_os = "linux")) {
        build.define("RENDERDOC_PLATFORM_LINUX", None);
        println!("cargo:rustc-link-lib=dl");

        if let Some(lib_dir) = find_renderdoc_so_dir() {
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
            println!("cargo:rustc-link-lib=dylib=renderdoc");
        }
    } else if cfg!(target_os = "macos") {
        build.define("RENDERDOC_PLATFORM_APPLE", None);
    }

    build.compile("renderdog_replay");

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/replay.cc");
    println!("cargo:rerun-if-changed=include/replay.h");
}

#[cfg(windows)]
fn find_renderdoc_lib_dir() -> PathBuf {
    let candidates = [
        env::var_os("RENDERDOG_REPLAY_RENDERDOC_LIB_DIR").map(PathBuf::from),
        env::var_os("RENDERDOG_RENDERDOC_DIR").map(PathBuf::from),
    ];

    for base in candidates.into_iter().flatten() {
        let direct = base.join("renderdoc.lib");
        if direct.exists() {
            return base;
        }

        for sub in ["lib", "Lib", "x64", "build", "Build"] {
            let dir = base.join(sub);
            if dir.join("renderdoc.lib").exists() {
                return dir;
            }
        }
    }

    panic!(
        "renderdog-replay (cxx-replay) requires linking against RenderDoc's import library.\n\
Set `RENDERDOG_REPLAY_RENDERDOC_LIB_DIR` or `RENDERDOG_RENDERDOC_DIR` to a directory containing `renderdoc.lib`."
    );
}

#[cfg(not(windows))]
fn find_renderdoc_lib_dir() -> PathBuf {
    unreachable!("windows-only")
}

#[cfg(all(unix, target_os = "linux"))]
fn find_renderdoc_so_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("RENDERDOG_REPLAY_RENDERDOC_LIB_DIR").map(PathBuf::from) {
        return Some(dir);
    }
    if let Some(dir) = env::var_os("RENDERDOG_RENDERDOC_DIR").map(PathBuf::from) {
        return Some(dir);
    }
    None
}

#[cfg(not(all(unix, target_os = "linux")))]
fn find_renderdoc_so_dir() -> Option<PathBuf> {
    None
}
