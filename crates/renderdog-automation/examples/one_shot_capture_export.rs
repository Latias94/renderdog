use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "Usage:\n  cargo run -p renderdog-automation --example one_shot_capture_export -- <exe> [args...]\n\
             \nEnvironment:\n  RENDERDOG_RENDERDOC_DIR=<RenderDoc install root>"
        );
        std::process::exit(2);
    }

    let (executable, exe_args) = args.split_first().expect("checked above");

    let install = renderdog::RenderDocInstallation::detect()?;

    let env_diag = install.diagnose_environment().ok();
    if let Some(diag) = &env_diag {
        for w in &diag.warnings {
            eprintln!("warning: {w}");
        }
    }

    let cwd = std::env::current_dir()?;
    let res = install.capture_and_export_bundle_jsonl(
        &cwd,
        &renderdog::CaptureAndExportBundleRequest {
            target: renderdog::LaunchCaptureRequest {
                executable: executable.clone(),
                args: exe_args.to_vec(),
                working_dir: None,
                artifacts_dir: None,
                capture_template_name: Some("capture_{app}_{timestamp}_{frame}".to_string()),
            },
            capture: renderdog::OneShotCaptureOptions::default(),
            output: renderdog::ExportOutput::default(),
            drawcall_scope: renderdog::DrawcallScope::default(),
            filter: renderdog::EventFilter::default(),
            bindings: renderdog::BindingsExportOptions::default(),
            post_actions: renderdog::CapturePostActions::default(),
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&res)?);

    Ok(())
}
