use std::path::PathBuf;

use renderdog_replay_experimental::Replay;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let capture = args.next().ok_or_else(|| {
        anyhow::anyhow!("usage: replay_pick_pixel <capture.rdc> [texture_index] [x] [y] [out.png]")
    })?;

    let texture_index: u32 = args.next().unwrap_or_else(|| "0".to_string()).parse()?;
    let x: u32 = args.next().unwrap_or_else(|| "0".to_string()).parse()?;
    let y: u32 = args.next().unwrap_or_else(|| "0".to_string()).parse()?;
    let out = args.next().map(PathBuf::from);

    let mut replay = Replay::new(None).map_err(|e| anyhow::anyhow!("{e}"))?;
    replay
        .open_capture(&capture)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let textures = replay
        .list_textures_json()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("textures: {textures}");

    let pix = replay
        .pick_pixel(texture_index, x, y)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("pixel: r={} g={} b={} a={}", pix.r, pix.g, pix.b, pix.a);

    if let Some(out) = out {
        replay
            .save_texture_png(texture_index, out.to_string_lossy().as_ref())
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("saved: {}", out.display());
    }

    Ok(())
}
