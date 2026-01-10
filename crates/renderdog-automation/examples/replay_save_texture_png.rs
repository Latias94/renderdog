use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: replay_save_texture_png <capture.rdc> <texture_index> <output.png> [event_id]"
        )
    })?;
    let texture_index: u32 = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing texture_index"))?
        .parse()?;
    let output_path = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing output_path"))?;
    let event_id = args.next().map(|s| s.parse()).transpose()?;

    let install = renderdog::RenderDocInstallation::detect()?;
    let cwd = std::env::current_dir()?;

    let res = install.replay_save_texture_png(
        &cwd,
        &renderdog::ReplaySaveTexturePngRequest {
            capture_path,
            event_id,
            texture_index,
            output_path,
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
