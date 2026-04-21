use std::path::PathBuf;

use renderdog_automation as renderdog;

fn parse_selection_arg(
    arg: Option<String>,
) -> anyhow::Result<(renderdog::ReplayEventSelector, Option<PathBuf>)> {
    let Some(value) = arg else {
        return Ok((renderdog::ReplayEventSelector::last_drawcall(), None));
    };

    if value == "last_drawcall" {
        return Ok((renderdog::ReplayEventSelector::last_drawcall(), None));
    }

    if let Some(value) = value.strip_prefix("event:") {
        let event_id = value
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("invalid event selector: event:<id>"))?;
        return Ok((renderdog::ReplayEventSelector::event_id(event_id), None));
    }

    if let Ok(event_id) = value.parse::<u32>() {
        return Ok((renderdog::ReplayEventSelector::event_id(event_id), None));
    }

    Ok((
        renderdog::ReplayEventSelector::last_drawcall(),
        Some(PathBuf::from(value)),
    ))
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: replay_save_outputs_png <capture.rdc> [last_drawcall|event:<id>] [out_dir] [basename]"
        )
    })?;

    let (selection, selection_out_dir) = parse_selection_arg(args.next())?;

    let cwd = std::env::current_dir()?;
    let out_dir = selection_out_dir.or_else(|| args.next().map(PathBuf::from));
    let basename = args.next();

    let install = renderdog::RenderDocInstallation::detect()?;

    let res = install.replay_save_outputs_png(
        &cwd,
        &renderdog::ReplaySaveOutputsPngRequest {
            capture: renderdog::CaptureInput { capture_path },
            selection,
            output: renderdog::ExportOutput {
                output_dir: out_dir.map(|path| path.display().to_string()),
                basename,
            },
            include_depth: false,
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
