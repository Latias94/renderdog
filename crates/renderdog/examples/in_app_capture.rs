use renderdog::{InputButton, RenderDog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rd = RenderDog::new()?;

    let (major, minor, patch) = rd.get_api_version()?;
    println!("RenderDoc API: {major}.{minor}.{patch}");
    println!("Requested version: {:?}", rd.requested_version());

    rd.set_capture_keys(&[InputButton::F12])?;
    rd.set_focus_toggle_keys(&[InputButton::F11])?;

    println!("Capture key set to F12. Focus toggle key set to F11.");
    println!("Triggering capture...");
    rd.trigger_capture()?;

    Ok(())
}
