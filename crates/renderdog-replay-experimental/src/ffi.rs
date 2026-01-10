#[cfg(feature = "cxx-replay")]
#[cxx::bridge(namespace = "renderdog::replay")]
mod cxx_ffi {
    struct PixelRgba {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }

    unsafe extern "C++" {
        include!("replay.h");

        type ReplaySession;

        fn replay_session_new(renderdoc_path: &str) -> Result<UniquePtr<ReplaySession>>;
        fn open_capture(self: Pin<&mut ReplaySession>, capture_path: &str) -> Result<()>;
        fn set_frame_event(self: Pin<&mut ReplaySession>, event_id: u32) -> Result<()>;
        fn list_textures_json(self: &ReplaySession) -> Result<String>;
        fn pick_pixel(
            self: &ReplaySession,
            texture_index: u32,
            x: u32,
            y: u32,
        ) -> Result<PixelRgba>;
        fn save_texture_png(
            self: &ReplaySession,
            texture_index: u32,
            output_path: &str,
        ) -> Result<()>;
    }
}

#[cfg(feature = "cxx-replay")]
pub use cxx_ffi::*;
