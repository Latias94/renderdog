mod ffi;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("`renderdog-replay` requires the `cxx-replay` feature")]
    FeatureNotEnabled,

    #[error("pick_pixel returned {0} values (expected 4)")]
    InvalidPickPixelReturnLen(usize),

    #[cfg(feature = "cxx-replay")]
    #[error(transparent)]
    Cxx(#[from] cxx::Exception),
}

#[cfg(feature = "cxx-replay")]
pub struct Replay {
    inner: cxx::UniquePtr<ffi::cxx_ffi::ReplaySession>,
}

#[cfg(feature = "cxx-replay")]
impl Replay {
    pub fn new(renderdoc_path: Option<&str>) -> Result<Self, ReplayError> {
        let path = renderdoc_path.unwrap_or("");
        let inner = ffi::cxx_ffi::replay_session_new(path)?;
        Ok(Self { inner })
    }

    pub fn open_capture(&mut self, capture_path: &str) -> Result<(), ReplayError> {
        let pinned = self.inner.pin_mut();
        pinned.open_capture(capture_path)?;
        Ok(())
    }

    pub fn set_frame_event(&mut self, event_id: u32) -> Result<(), ReplayError> {
        let pinned = self.inner.pin_mut();
        pinned.set_frame_event(event_id)?;
        Ok(())
    }

    pub fn list_textures_json(&self) -> Result<String, ReplayError> {
        Ok(self.inner.list_textures_json()?)
    }

    pub fn pick_pixel(&self, texture_index: u32, x: u32, y: u32) -> Result<[f32; 4], ReplayError> {
        let v = self.inner.pick_pixel(texture_index, x, y)?;
        if v.len() != 4 {
            return Err(ReplayError::InvalidPickPixelReturnLen(v.len()));
        }
        Ok([v[0], v[1], v[2], v[3]])
    }

    pub fn save_texture_png(
        &self,
        texture_index: u32,
        output_path: &str,
    ) -> Result<(), ReplayError> {
        self.inner.save_texture_png(texture_index, output_path)?;
        Ok(())
    }
}

#[cfg(not(feature = "cxx-replay"))]
pub struct Replay;

#[cfg(not(feature = "cxx-replay"))]
impl Replay {
    pub fn new(_renderdoc_path: Option<&str>) -> Result<Self, ReplayError> {
        Err(ReplayError::FeatureNotEnabled)
    }
}
