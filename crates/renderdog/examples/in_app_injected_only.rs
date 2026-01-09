use renderdog::{InAppError, RenderDog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match RenderDog::connect_injected() {
        Ok(rd) => {
            let (major, minor, patch) = rd.get_api_version()?;
            println!("Connected to injected RenderDoc. API: {major}.{minor}.{patch}");
            Ok(())
        }
        Err(InAppError::NotAvailable) => {
            eprintln!(
                "RenderDoc is not available in the current process.\n\
                - On Windows, this usually means the target wasn't launched via `renderdoccmd capture`.\n\
                - If you want to load RenderDoc dynamically, use the `in_app_capture` example instead."
            );
            Ok(())
        }
        Err(e) => Err(Box::new(e)),
    }
}
