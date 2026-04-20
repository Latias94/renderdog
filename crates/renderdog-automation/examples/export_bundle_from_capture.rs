use std::path::PathBuf;

use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args.next().ok_or_else(|| {
        anyhow::anyhow!("usage: export_bundle_from_capture <capture.rdc> [out_dir] [basename]")
    })?;

    let cwd = std::env::current_dir()?;
    let out_dir = args.next().map(PathBuf::from);
    let basename = args.next();

    let install = renderdog::RenderDocInstallation::detect()?;

    let res = install.export_bundle_jsonl(
        &cwd,
        &renderdog::ExportBundleRequest {
            capture: renderdog::CaptureInput { capture_path },
            output: renderdog::ExportOutput {
                output_dir: out_dir.map(|path| path.display().to_string()),
                basename,
            },
            bundle: renderdog::BundleExportOptions {
                drawcall_scope: renderdog::DrawcallScope::default(),
                filter: renderdog::EventFilter::default(),
                bindings: renderdog::BindingsExportOptions::default(),
                post_actions: renderdog::CapturePostActions::default(),
            },
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
