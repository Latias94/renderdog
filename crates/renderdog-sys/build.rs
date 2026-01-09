use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let pregenerated = manifest_dir.join("src").join("bindings_pregenerated.rs");
    let vendor_header = manifest_dir.join("vendor").join("renderdoc_app.h");
    let submodule_header = workspace_root(&manifest_dir)
        .join("third-party")
        .join("renderdoc")
        .join("renderdoc")
        .join("api")
        .join("app")
        .join("renderdoc_app.h");

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("build.rs").display()
    );
    println!("cargo:rerun-if-changed={}", pregenerated.display());
    println!("cargo:rerun-if-changed={}", vendor_header.display());
    println!("cargo:rerun-if-changed={}", submodule_header.display());
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_HEADER");
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_REGEN_BINDINGS");
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_VERBOSE");

    let out_bindings = out_dir.join("bindings.rs");

    // docs.rs should never require a native toolchain (clang/libclang).
    let docs_rs = env::var("DOCS_RS").is_ok();
    let regen = !docs_rs && parse_bool_env("RENDERDOG_SYS_REGEN_BINDINGS");
    let feature_bindgen = env::var("CARGO_FEATURE_BINDGEN").is_ok();
    let verbose = parse_bool_env("RENDERDOG_SYS_VERBOSE");

    if regen {
        if !feature_bindgen {
            panic!(
                "RENDERDOG_SYS_REGEN_BINDINGS is set but feature `bindgen` is not enabled. \
                 Enable it via: cargo build -p renderdog-sys --features bindgen"
            );
        }

        generate_bindings(&manifest_dir, &out_bindings);
        sanitize_bindings_file(&out_bindings);
    } else {
        if docs_rs && env::var_os("RENDERDOG_SYS_REGEN_BINDINGS").is_some() {
            println!("cargo:warning=DOCS_RS detected: ignoring RENDERDOG_SYS_REGEN_BINDINGS");
        }

        let content = fs::read_to_string(&pregenerated).expect("read pregenerated bindings");
        fs::write(&out_bindings, sanitize_bindings_string(&content))
            .expect("write pregenerated bindings");
        if verbose {
            println!(
                "cargo:warning=Using pregenerated bindings: {}",
                pregenerated.display()
            );
        }
    }
}

#[cfg(feature = "bindgen")]
fn generate_bindings(manifest_dir: &Path, out: &Path) {
    let header = select_header_path(manifest_dir);
    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("RENDERDOC_.*")
        .allowlist_var("eRENDERDOC_.*")
        .allowlist_function("RENDERDOC_GetAPI")
        .allowlist_type("pRENDERDOC_.*")
        .derive_default(true)
        .derive_debug(true)
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings for renderdoc_app.h");

    bindings
        .write_to_file(out)
        .expect("Couldn't write renderdog bindings!");
}

#[cfg(not(feature = "bindgen"))]
fn generate_bindings(_manifest_dir: &Path, _out: &Path) {
    panic!("bindgen feature is not enabled");
}

#[cfg(feature = "bindgen")]
fn select_header_path(manifest_dir: &Path) -> PathBuf {
    // 1) Explicit override (recommended for CI/maintainers)
    if let Ok(p) = env::var("RENDERDOG_SYS_HEADER") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return pb;
        }
        panic!(
            "RENDERDOG_SYS_HEADER is set but not a file: {}",
            pb.display()
        );
    }

    // 2) If RenderDoc is vendored as a git submodule, prefer the authoritative header.
    // This matches the layout of the official RenderDoc repo.
    let submodule = workspace_root(manifest_dir)
        .join("third-party")
        .join("renderdoc")
        .join("renderdoc")
        .join("api")
        .join("app")
        .join("renderdoc_app.h");
    if submodule.is_file() {
        return submodule;
    }

    // 3) Fallback: minimal vendored header for regeneration.
    manifest_dir.join("vendor").join("renderdoc_app.h")
}

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..").join("..")
}

fn parse_bool_env(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
            "" => false,
            "1" | "true" | "yes" | "y" | "on" => true,
            "0" | "false" | "no" | "n" | "off" => false,
            _ => true,
        },
        Err(_) => false,
    }
}

fn sanitize_bindings_file(path: &Path) {
    if let Ok(content) = fs::read_to_string(path) {
        let sanitized = sanitize_bindings_string(&content);
        let _ = fs::write(path, sanitized);
    }
}

fn sanitize_bindings_string(content: &str) -> String {
    // Strip inner attributes like #![allow(...)] that bindgen may emit.
    let mut out = String::with_capacity(content.len());
    let mut skip_next_blank = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#![") {
            skip_next_blank = true;
            continue;
        }
        if skip_next_blank {
            if trimmed.is_empty() {
                continue;
            }
            skip_next_blank = false;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
