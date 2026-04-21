use renderdog_automation as renderdog;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture_path = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: qrenderdoc_replay_pick_pixel <capture.rdc> <texture_index> <x> <y> [event_id]"
        )
    })?;
    let texture_index: u32 = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing texture_index"))?
        .parse()?;
    let x: u32 = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing x"))?
        .parse()?;
    let y: u32 = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing y"))?
        .parse()?;
    let event_id = args.next().map(|s| s.parse()).transpose()?;

    let install = renderdog::RenderDocInstallation::detect()?;
    let cwd = std::env::current_dir()?;

    let res = install.replay_pick_pixel(
        &cwd,
        &renderdog::ReplayPickPixelRequest {
            replay: renderdog::ReplayTextureRequest {
                context: renderdog::ReplayRequestContext {
                    capture: renderdog::CaptureInput { capture_path },
                    event_id,
                },
                texture: renderdog::ReplayTextureRef { texture_index },
            },
            pixel: renderdog::PixelPosition { x, y },
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
