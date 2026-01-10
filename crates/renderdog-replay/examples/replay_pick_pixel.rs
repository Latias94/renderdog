use std::path::PathBuf;

use renderdog_replay::Replay;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);

    let mut renderdoc_path: Option<String> = None;
    let capture = loop {
        let Some(arg) = args.next() else {
            return Err(anyhow::anyhow!(
                "usage: replay_pick_pixel [--renderdoc=<path-to-renderdoc.dll|librenderdoc.so>] <capture.rdc> [texture_index] [x] [y] [out.png]"
            ));
        };

        if let Some(v) = arg.strip_prefix("--renderdoc=") {
            renderdoc_path = Some(v.to_string());
            continue;
        }

        break arg;
    };

    let texture_index: u32 = args.next().unwrap_or_else(|| "0".to_string()).parse()?;
    let x: u32 = args.next().unwrap_or_else(|| "0".to_string()).parse()?;
    let y: u32 = args.next().unwrap_or_else(|| "0".to_string()).parse()?;
    let out = args.next().map(PathBuf::from);

    let mut replay = Replay::new(renderdoc_path.as_deref()).map_err(|e| anyhow::anyhow!("{e}"))?;
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
    println!("pixel: r={} g={} b={} a={}", pix[0], pix[1], pix[2], pix[3]);

    if let Some(out) = out {
        replay
            .save_texture_png(texture_index, out.to_string_lossy().as_ref())
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("saved: {}", out.display());
    }

    Ok(())
}
