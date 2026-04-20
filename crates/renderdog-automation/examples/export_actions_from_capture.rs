use std::path::PathBuf;

use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args.next().ok_or_else(|| {
        anyhow::anyhow!("usage: export_actions_from_capture <capture.rdc> [out_dir] [basename]")
    })?;

    let cwd = std::env::current_dir()?;
    let out_dir = args.next().map(PathBuf::from);
    let basename = args.next();

    let install = renderdog::RenderDocInstallation::detect()?;

    let res = install.export_actions_jsonl(
        &cwd,
        &renderdog::ExportActionsRequest {
            capture: renderdog::CaptureInput { capture_path },
            output: renderdog::ExportOutput {
                output_dir: out_dir.map(|path| path.display().to_string()),
                basename,
            },
            drawcall_scope: renderdog::DrawcallScope::default(),
            filter: renderdog::EventFilter::default(),
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
