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
            capture_path,
            only_drawcalls: true,
            marker_prefix: None,
            event_id_min: None,
            event_id_max: None,
            name_contains: None,
            marker_contains,
            case_sensitive: false,
            max_results: Some(200),
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
