use std::path::PathBuf;

use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args.next().ok_or_else(|| {
        anyhow::anyhow!("usage: export_actions_from_capture <capture.rdc> [out_dir] [basename]")
    })?;

    let cwd = std::env::current_dir()?;
    let out_dir = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| renderdog::default_exports_dir(&cwd));
    std::fs::create_dir_all(&out_dir)?;

    let basename = args.next().unwrap_or_else(|| {
        PathBuf::from(&capture_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("capture")
            .to_string()
    });

    let install = renderdog::RenderDocInstallation::detect()?;

    let res = install.export_actions_jsonl(
        &cwd,
        &renderdog::ExportActionsRequest {
            capture_path,
            output_dir: out_dir.display().to_string(),
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

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
