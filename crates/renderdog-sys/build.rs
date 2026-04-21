use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
struct RenderDocSource {
    header_path: PathBuf,
    workspace_replay_version: String,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let pregenerated = manifest_dir.join("src").join("bindings_pregenerated.rs");
    let vendor_header = manifest_dir.join("vendor").join("renderdoc_app.h");
    let vendor_replay_version = manifest_dir
        .join("vendor")
        .join("renderdoc_replay_version.txt");
    let submodule_header = workspace_root(&manifest_dir)
        .join("third-party")
        .join("renderdoc")
        .join("renderdoc")
        .join("api")
        .join("app")
        .join("renderdoc_app.h");
    let submodule_replay_version_header = workspace_root(&manifest_dir)
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
    println!("cargo:rerun-if-changed={}", pregenerated.display());
    println!("cargo:rerun-if-changed={}", vendor_header.display());
    println!("cargo:rerun-if-changed={}", vendor_replay_version.display());
    println!("cargo:rerun-if-changed={}", submodule_header.display());
    println!(
        "cargo:rerun-if-changed={}",
        submodule_replay_version_header.display()
    );
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_HEADER");
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_REPLAY_VERSION_HEADER");
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_REGEN_BINDINGS");
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_VERBOSE");

    let out_bindings = out_dir.join("bindings.rs");
    let renderdoc_source = select_renderdoc_source(
        &manifest_dir,
        &vendor_header,
        &vendor_replay_version,
        &submodule_header,
        &submodule_replay_version_header,
    );

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

        generate_bindings(&renderdoc_source.header_path, &out_bindings);
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

    let workspace_replay_version = renderdoc_source.workspace_replay_version.clone();
    println!("cargo:rustc-env=RENDERDOG_SYS_WORKSPACE_REPLAY_VERSION={workspace_replay_version}");
}

#[cfg(feature = "bindgen")]
fn generate_bindings(header: &Path, out: &Path) {
    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Recent RenderDoc versions expose annotation types that use `bool` in the C API.
        // Clang needs `stdbool.h` preincluded when parsing the header as C.
        .clang_arg("-include")
        .clang_arg("stdbool.h")
        // Keep enums "safe-ish" for FFI (as integer newtypes) while still providing ergonomic
        // associated constants (e.g. `RENDERDOC_InputButton::eRENDERDOC_Key_F12`).
        //
        // The default enum style in bindgen is `Consts`, which can change the emitted symbol
        // names depending on bindgen versions/config, and would break downstream crates.
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .allowlist_type("RENDERDOC_.*")
        .allowlist_var("eRENDERDOC_.*")
        .allowlist_var("RENDERDOC_.*")
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
fn generate_bindings(_header: &Path, _out: &Path) {
    panic!("bindgen feature is not enabled");
}

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..").join("..")
}

fn select_renderdoc_source(
    manifest_dir: &Path,
    vendor_header: &Path,
    vendor_replay_version: &Path,
    submodule_header: &Path,
    submodule_replay_version_header: &Path,
) -> RenderDocSource {
    if let Some(header_path) = explicit_header_path() {
        let replay_version_header = explicit_replay_version_header_path(&header_path);
        return RenderDocSource {
            header_path,
            workspace_replay_version: read_replay_version_header(&replay_version_header),
        };
    }

    if submodule_header.is_file() {
        return RenderDocSource {
            header_path: submodule_header.to_path_buf(),
            workspace_replay_version: read_replay_version_header(submodule_replay_version_header),
        };
    }

    RenderDocSource {
        header_path: vendor_header.to_path_buf(),
        workspace_replay_version: read_replay_version_file(vendor_replay_version, manifest_dir),
    }
}

fn explicit_header_path() -> Option<PathBuf> {
    let p = env::var_os("RENDERDOG_SYS_HEADER")?;
    let path = PathBuf::from(p);
    if path.is_file() {
        return Some(path);
    }
    panic!(
        "RENDERDOG_SYS_HEADER is set but not a file: {}",
        path.display()
    );
}

fn explicit_replay_version_header_path(header_path: &Path) -> PathBuf {
    if let Some(path) = env::var_os("RENDERDOG_SYS_REPLAY_VERSION_HEADER").map(PathBuf::from) {
        if path.is_file() {
            return path;
        }
        panic!(
            "RENDERDOG_SYS_REPLAY_VERSION_HEADER is set but not a file: {}",
            path.display()
        );
    }

    if let Some(path) = infer_replay_version_header_path(header_path)
        && path.is_file()
    {
        return path;
    }

    panic!(
        "RENDERDOG_SYS_HEADER is set to {}, but no matching replay version header could be inferred. \
         Set RENDERDOG_SYS_REPLAY_VERSION_HEADER to the corresponding renderdoc/api/replay/version.h.",
        header_path.display()
    );
}

fn infer_replay_version_header_path(header_path: &Path) -> Option<PathBuf> {
    let app_dir = header_path.parent()?;
    let api_dir = app_dir.parent()?;
    Some(api_dir.join("replay").join("version.h"))
}

fn read_replay_version_header(path: &Path) -> String {
    let content = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!(
            "failed to read RenderDoc replay version header at {}: {err}",
            path.display()
        )
    });
    parse_replay_version_header(&content).unwrap_or_else(|| {
        panic!(
            "failed to parse RenderDoc replay version header at {}",
            path.display()
        )
    })
}

fn read_replay_version_file(path: &Path, manifest_dir: &Path) -> String {
    let version = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!(
            "failed to read vendored replay version at {} (workspace root: {}): {err}",
            path.display(),
            workspace_root(manifest_dir).display()
        )
    });
    let version = version.trim();
    if version.is_empty() {
        panic!("vendored replay version file is empty: {}", path.display());
    }
    version.to_string()
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
