use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: replay_list_textures <capture.rdc> [event_id]"))?;
    let event_id = args.next().map(|s| s.parse()).transpose()?;

    let install = renderdog::RenderDocInstallation::detect()?;
    let cwd = std::env::current_dir()?;

    let res = install.replay_list_textures(
        &cwd,
        &renderdog::ReplayListTexturesRequest {
            capture_path,
            event_id,
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
