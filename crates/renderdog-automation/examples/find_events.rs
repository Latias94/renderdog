use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: find_events <capture.rdc> [marker_contains]"))?;

    let marker_contains = args.next();

    let install = renderdog::RenderDocInstallation::detect()?;
    let cwd = std::env::current_dir()?;

    let res = install.find_events(
        &cwd,
        &renderdog::FindEventsRequest {
            capture: renderdog::CaptureInput { capture_path },
            drawcall_scope: renderdog::DrawcallScope {
                only_drawcalls: true,
            },
            filter: renderdog::EventFilter {
                marker_contains,
                ..Default::default()
            },
            limit: renderdog::FindEventsLimit {
                max_results: Some(200),
            },
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
