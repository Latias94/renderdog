#![allow(dead_code)]

#[cxx::bridge(namespace = "renderdog::replay")]
pub(crate) mod cxx_ffi {
    unsafe extern "C++" {
        include!("replay.h");

        type ReplaySession;

        fn replay_session_new(renderdoc_path: &str) -> Result<UniquePtr<ReplaySession>>;
        fn open_capture(self: Pin<&mut ReplaySession>, capture_path: &str) -> Result<()>;
        fn set_frame_event(self: Pin<&mut ReplaySession>, event_id: u32) -> Result<()>;
        fn list_textures_json(self: &ReplaySession) -> Result<String>;
        fn pick_pixel(self: &ReplaySession, texture_index: u32, x: u32, y: u32)
        -> Result<Vec<f32>>;
        fn save_texture_png(
            self: &ReplaySession,
            texture_index: u32,
            output_path: &str,
        ) -> Result<()>;
    }
}
