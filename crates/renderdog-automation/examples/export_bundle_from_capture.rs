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
            capture_path,
            output_dir: out_dir.map(|path| path.display().to_string()),
            basename,
            only_drawcalls: false,
            marker_prefix: None,
            event_id_min: None,
            event_id_max: None,
            name_contains: None,
            marker_contains: None,
            case_sensitive: false,
            include_cbuffers: false,
            include_outputs: false,
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
