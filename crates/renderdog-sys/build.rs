use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
struct RenderDocSource {
    bindings_header_path: Option<PathBuf>,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let pregenerated = manifest_dir.join("src").join("bindings_pregenerated.rs");
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
    println!("cargo:rerun-if-changed={}", submodule_header.display());
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_HEADER");
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_REGEN_BINDINGS");
    println!("cargo:rerun-if-env-changed=RENDERDOG_SYS_VERBOSE");

    let out_bindings = out_dir.join("bindings.rs");
    let renderdoc_source = select_renderdoc_source(&submodule_header);

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
        let header = renderdoc_source
            .bindings_header_path
            .as_deref()
            .unwrap_or_else(|| {
                panic!(
                    "RENDERDOG_SYS_REGEN_BINDINGS requires either the `third-party/renderdoc` \
                     submodule or `RENDERDOG_SYS_HEADER=/path/to/renderdoc_app.h`."
                )
            });

        generate_bindings(header, &out_bindings);
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

fn select_renderdoc_source(submodule_header: &Path) -> RenderDocSource {
    if let Some(header_path) = explicit_header_path() {
        return RenderDocSource {
            bindings_header_path: Some(header_path),
        };
    }

    if submodule_header.is_file() {
        return RenderDocSource {
            bindings_header_path: Some(submodule_header.to_path_buf()),
        };
    }

    RenderDocSource {
        bindings_header_path: None,
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
