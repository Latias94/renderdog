use renderdog::{CaptureOption, InputButton, OverlayBits, RenderDog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rd = RenderDog::new()?;

    let (major, minor, patch) = rd.get_api_version()?;
    println!("RenderDoc API: {major}.{minor}.{patch}");
    println!("Requested version: {:?}", rd.requested_version());

    // Configure capture output (template is interpreted by RenderDoc).
    // See RenderDoc docs for supported tokens such as {app} / {frame} / {timestamp}.
    rd.set_capture_file_path_template("artifacts/renderdoc/{app}_{timestamp}_{frame}.rdc")?;
    rd.set_capture_title("renderdog: in-app options/overlay example")?;
    rd.set_capture_file_comments(None, "Captured via renderdog in-app API example")?;

    // Capture options: set a few safe defaults.
    rd.set_capture_option_u32(CaptureOption::ApiValidation, 1)?;
    rd.set_capture_option_u32(CaptureOption::CaptureCallstacks, 1)?;
    rd.set_capture_option_u32(CaptureOption::DelayForDebugger, 0)?;

    // Overlay: keep it minimal (enabled + capture list).
    rd.mask_overlay_bits_flags(
        OverlayBits::ALL,
        OverlayBits::ENABLED | OverlayBits::CAPTURE_LIST,
    )?;

    // Optional: configure hotkeys for capture/focus toggle.
    rd.set_capture_keys(&[InputButton::F12])?;
    rd.set_focus_toggle_keys(&[InputButton::F11])?;

    println!("Configured capture template/options/overlay.");
    println!("Press F12 in your application (if supported) or call TriggerCapture in code.");

    Ok(())
}
