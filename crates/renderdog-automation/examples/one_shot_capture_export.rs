use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use renderdog_automation as renderdog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "Usage:\n  cargo run -p renderdog-automation --example one_shot_capture_export -- <exe> [args...]\n\
             \nEnvironment:\n  RENDERDOG_RENDERDOC_DIR=<RenderDoc install root>"
        );
        std::process::exit(2);
    }

    let executable = PathBuf::from(&args[0]);
    let exe_args: Vec<OsString> = args[1..].iter().cloned().map(OsString::from).collect();

    let install = renderdog::RenderDocInstallation::detect()?;

    let env_diag = install.diagnose_environment().ok();
    if let Some(diag) = &env_diag {
        for w in &diag.warnings {
            eprintln!("warning: {w}");
        }
    }

    let cwd = std::env::current_dir()?;
    let artifacts_dir = renderdog::default_artifacts_dir(&cwd);
    let exports_dir = renderdog::default_exports_dir(&cwd);
    std::fs::create_dir_all(&artifacts_dir)?;
    std::fs::create_dir_all(&exports_dir)?;

    let capture_template = artifacts_dir.join("capture_{app}_{timestamp}_{frame}.rdc");

    let launch = install.launch_capture(&renderdog::CaptureLaunchRequest {
        executable,
        args: exe_args,
        working_dir: None,
        capture_file_template: Some(capture_template.clone()),
    })?;
    eprintln!(
        "launched renderdoccmd capture: target_ident={}",
        launch.target_ident
    );

    let capture = install.trigger_capture_via_target_control(
        &cwd,
        &renderdog::TriggerCaptureRequest {
            host: "localhost".to_string(),
            target_ident: launch.target_ident,
            num_frames: 1,
            timeout_s: 60,
        },
    )?;
    eprintln!("captured: {}", capture.capture_path);

    let basename = Path::new(&capture.capture_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("capture")
        .to_string();

    let export = install.export_actions_jsonl(
        &cwd,
        &renderdog::ExportActionsRequest {
            capture_path: capture.capture_path,
            output_dir: exports_dir.display().to_string(),
            basename,
            only_drawcalls: false,
            marker_prefix: None,
            event_id_min: None,
            event_id_max: None,
            name_contains: None,
            marker_contains: None,
            case_sensitive: false,
        },
    )?;

    println!("actions_jsonl: {}", export.actions_jsonl_path);
    println!("summary_json:  {}", export.summary_json_path);
    println!(
        "actions: total={}, drawcalls={}",
        export.total_actions, export.drawcall_actions
    );

    Ok(())
}
