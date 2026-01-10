use std::{
    cell::Cell,
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    ptr::NonNull,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use renderdog_sys as sys;

use crate::OverlayBits;

#[cfg(unix)]
use libloading::Library;

#[cfg(windows)]
use windows_sys::Win32::Foundation::GetLastError;

#[cfg(windows)]
use windows_sys::Win32::Foundation::FreeLibrary;

#[cfg(windows)]
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};

pub use renderdog_sys::{
    RENDERDOC_CaptureOption, RENDERDOC_DevicePointer, RENDERDOC_InputButton, RENDERDOC_WindowHandle,
};

#[derive(Debug)]
enum LibraryGuard {
    #[cfg(windows)]
    WindowsBorrowed,
    #[cfg(windows)]
    WindowsOwned(isize),
    #[cfg(unix)]
    Unix {
        #[allow(dead_code)]
        _lib: Library,
    },
}

#[cfg(windows)]
impl Drop for LibraryGuard {
    fn drop(&mut self) {
        if let LibraryGuard::WindowsOwned(module) = self {
            unsafe {
                FreeLibrary(*module);
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum InAppError {
    #[error("renderdoc library is not available in the current process")]
    NotAvailable,

    #[error("failed to load renderdoc library (Win32 error {0})")]
    LoadLibraryFailed(u32),

    #[error("failed to load renderdoc library: {0}")]
    DynamicLoadFailed(String),

    #[error("RENDERDOC_GetAPI not found in renderdoc module")]
    MissingGetApi,

    #[error("RENDERDOC_GetAPI symbol load failed: {0}")]
    GetApiSymbolLoadFailed(String),

    #[error("RENDERDOC_GetAPI returned failure")]
    GetApiFailed,

    #[error("RENDERDOC_GetAPI failed for all requested versions")]
    GetApiFailedAllVersions,

    #[error("required API function pointer is null: {0}")]
    MissingFunction(&'static str),

    #[error("capture index out of range")]
    InvalidCaptureIndex,

    #[error("too many keys (max i32)")]
    TooManyKeys,

    #[error("invalid UTF-8 from RenderDoc")]
    InvalidUtf8,
}

pub struct RenderDocInApp {
    api: NonNull<sys::RENDERDOC_API_1_6_0>,
    _guard: LibraryGuard,
    requested_version: sys::RENDERDOC_Version,
    _not_sync: Cell<()>,
}

impl RenderDocInApp {
    pub fn try_connect() -> Result<Self, InAppError> {
        #[cfg(windows)]
        {
            // Only connect when already injected/loaded, do not LoadLibrary.
            let module = unsafe { GetModuleHandleA(c"renderdoc.dll".as_ptr().cast()) };
            if module == 0 {
                return Err(InAppError::NotAvailable);
            }

            Self::connect_from_windows_module(module, LibraryGuard::WindowsBorrowed)
        }

        #[cfg(not(windows))]
        {
            Err(InAppError::NotAvailable)
        }
    }

    pub fn try_load_and_connect(dll_path_or_name: &str) -> Result<Self, InAppError> {
        #[cfg(windows)]
        {
            let name = CString::new(dll_path_or_name).map_err(|_| InAppError::InvalidUtf8)?;
            let module = unsafe { LoadLibraryA(name.as_ptr() as *const u8) };
            if module == 0 {
                return Err(InAppError::LoadLibraryFailed(unsafe { GetLastError() }));
            }
            Self::connect_from_windows_module(module, LibraryGuard::WindowsOwned(module))
        }

        #[cfg(unix)]
        {
            // SAFETY: Dynamically loading a library is inherently unsafe. We only use this handle
            // to resolve RenderDoc's documented `RENDERDOC_GetAPI` symbol and then negotiate a
            // supported API version. Callers must ensure the path/name points to a compatible
            // RenderDoc library.
            let lib = unsafe { Library::new(dll_path_or_name) }
                .map_err(|e| InAppError::DynamicLoadFailed(e.to_string()))?;
            Self::connect_from_unix_library(lib)
        }

        #[cfg(not(any(windows, unix)))]
        {
            Err(InAppError::NotAvailable)
        }
    }

    pub fn try_connect_or_load(dll_path_or_name: &str) -> Result<Self, InAppError> {
        #[cfg(windows)]
        {
            if let Ok(name) = CString::new(dll_path_or_name) {
                let module = unsafe { GetModuleHandleA(name.as_ptr() as *const u8) };
                if module != 0 {
                    return Self::connect_from_windows_module(
                        module,
                        LibraryGuard::WindowsBorrowed,
                    );
                }
            }
        }

        Self::try_load_and_connect(dll_path_or_name)
    }

    pub fn try_load_and_connect_default() -> Result<Self, InAppError> {
        #[cfg(windows)]
        {
            Self::try_load_and_connect("renderdoc.dll")
        }

        #[cfg(unix)]
        {
            for candidate in ["librenderdoc.so", "librenderdoc.so.1"] {
                if let Ok(v) = Self::try_load_and_connect(candidate) {
                    return Ok(v);
                }
            }
            Err(InAppError::NotAvailable)
        }

        #[cfg(not(any(windows, unix)))]
        {
            Err(InAppError::NotAvailable)
        }
    }

    pub fn try_connect_or_load_default() -> Result<Self, InAppError> {
        #[cfg(windows)]
        {
            if let Ok(v) = Self::try_connect() {
                return Ok(v);
            }
        }

        Self::try_load_and_connect_default()
    }

    #[cfg(all(unix, target_os = "linux"))]
    pub fn try_connect_noload_default() -> Result<Self, InAppError> {
        use libloading::os::unix;

        // RTLD_NOLOAD is a non-POSIX extension; we only enable it on Linux.
        let flags = unix::RTLD_LAZY | unix::RTLD_LOCAL | libc::RTLD_NOLOAD;

        for candidate in ["librenderdoc.so", "librenderdoc.so.1"] {
            let lib = unsafe { unix::Library::open(Some(candidate), flags) };
            if let Ok(lib) = lib {
                return Self::connect_from_unix_library(Library::from(lib));
            }
        }

        Err(InAppError::NotAvailable)
    }

    #[cfg(all(unix, target_os = "linux"))]
    pub fn try_connect_noload_or_load_default() -> Result<Self, InAppError> {
        if let Ok(v) = Self::try_connect_noload_default() {
            return Ok(v);
        }
        Self::try_load_and_connect_default()
    }

    fn resolve_api(
        get_api: sys::pRENDERDOC_GetAPI,
    ) -> Result<(NonNull<sys::RENDERDOC_API_1_6_0>, sys::RENDERDOC_Version), InAppError> {
        let preferred = [
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_6_0,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_5_0,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_4_2,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_4_1,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_4_0,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_3_0,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_2_0,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_1_2,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_1_1,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_1_0,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_0_2,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_0_1,
            sys::RENDERDOC_Version::eRENDERDOC_API_Version_1_0_0,
        ];

        for version in preferred {
            let mut out: *mut std::ffi::c_void = std::ptr::null_mut();
            let ok = unsafe { get_api(version, &mut out) };
            if ok == 1 && !out.is_null() {
                let api = NonNull::new(out as *mut sys::RENDERDOC_API_1_6_0).unwrap();
                return Ok((api, version));
            }
        }

        Err(InAppError::GetApiFailedAllVersions)
    }

    #[cfg(windows)]
    fn connect_from_windows_module(module: isize, guard: LibraryGuard) -> Result<Self, InAppError> {
        let proc = unsafe { GetProcAddress(module, c"RENDERDOC_GetAPI".as_ptr().cast()) };
        if proc.is_none() {
            return Err(InAppError::MissingGetApi);
        }

        let get_api: sys::pRENDERDOC_GetAPI = unsafe { std::mem::transmute(proc.unwrap()) };
        let (api, requested_version) = Self::resolve_api(get_api)?;

        Ok(Self {
            api,
            _guard: guard,
            requested_version,
            _not_sync: Cell::new(()),
        })
    }

    #[cfg(unix)]
    fn connect_from_unix_library(lib: Library) -> Result<Self, InAppError> {
        let get_api = unsafe { lib.get::<sys::pRENDERDOC_GetAPI>(b"RENDERDOC_GetAPI\0") }
            .map_err(|e| InAppError::GetApiSymbolLoadFailed(e.to_string()))?;
        let get_api = *get_api;
        let (api, requested_version) = Self::resolve_api(get_api)?;

        Ok(Self {
            api,
            _guard: LibraryGuard::Unix { _lib: lib },
            requested_version,
            _not_sync: Cell::new(()),
        })
    }

    fn api(&self) -> &sys::RENDERDOC_API_1_6_0 {
        unsafe { self.api.as_ref() }
    }

    pub fn requested_version(&self) -> sys::RENDERDOC_Version {
        self.requested_version
    }

    pub fn get_api_version(&self) -> Result<(i32, i32, i32), InAppError> {
        let f = self
            .api()
            .GetAPIVersion
            .ok_or(InAppError::MissingFunction("GetAPIVersion"))?;
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;
        unsafe { f(&mut major, &mut minor, &mut patch) };
        Ok((major, minor, patch))
    }

    pub fn set_capture_file_path_template(&self, template: &str) -> Result<(), InAppError> {
        let f = unsafe { self.api().__bindgen_anon_2.SetCaptureFilePathTemplate }
            .ok_or(InAppError::MissingFunction("SetCaptureFilePathTemplate"))?;
        let s = CString::new(template).map_err(|_| InAppError::InvalidUtf8)?;
        unsafe { f(s.as_ptr()) };
        Ok(())
    }

    pub fn get_capture_file_path_template(&self) -> Result<String, InAppError> {
        let f = unsafe { self.api().__bindgen_anon_3.GetCaptureFilePathTemplate }
            .ok_or(InAppError::MissingFunction("GetCaptureFilePathTemplate"))?;
        let ptr = unsafe { f() };
        if ptr.is_null() {
            return Ok(String::new());
        }
        let s = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .map_err(|_| InAppError::InvalidUtf8)?;
        Ok(s.to_string())
    }

    pub fn get_capture_file_path_template_path(&self) -> Result<PathBuf, InAppError> {
        Ok(PathBuf::from(self.get_capture_file_path_template()?))
    }

    pub fn set_capture_file_path_template_path<P: AsRef<Path>>(
        &self,
        template: P,
    ) -> Result<(), InAppError> {
        let s = template.as_ref().to_str().ok_or(InAppError::InvalidUtf8)?;
        self.set_capture_file_path_template(s)
    }

    #[deprecated(since = "0.1.0", note = "renamed to get_capture_file_path_template")]
    pub fn get_log_file_path_template(&self) -> Result<String, InAppError> {
        self.get_capture_file_path_template()
    }

    #[deprecated(since = "0.1.0", note = "renamed to set_capture_file_path_template")]
    pub fn set_log_file_path_template(&self, template: &str) -> Result<(), InAppError> {
        self.set_capture_file_path_template(template)
    }

    #[deprecated(
        since = "0.1.0",
        note = "renamed to get_capture_file_path_template_path"
    )]
    pub fn get_log_file_path_template_path(&self) -> Result<PathBuf, InAppError> {
        self.get_capture_file_path_template_path()
    }

    #[deprecated(
        since = "0.1.0",
        note = "renamed to set_capture_file_path_template_path"
    )]
    pub fn set_log_file_path_template_path<P: AsRef<Path>>(
        &self,
        template: P,
    ) -> Result<(), InAppError> {
        self.set_capture_file_path_template_path(template)
    }

    pub fn set_capture_option_u32(
        &self,
        opt: impl Into<sys::RENDERDOC_CaptureOption>,
        val: u32,
    ) -> Result<bool, InAppError> {
        let f = self
            .api()
            .SetCaptureOptionU32
            .ok_or(InAppError::MissingFunction("SetCaptureOptionU32"))?;
        Ok(unsafe { f(opt.into(), val) } == 1)
    }

    pub fn set_capture_option_f32(
        &self,
        opt: impl Into<sys::RENDERDOC_CaptureOption>,
        val: f32,
    ) -> Result<bool, InAppError> {
        let f = self
            .api()
            .SetCaptureOptionF32
            .ok_or(InAppError::MissingFunction("SetCaptureOptionF32"))?;
        Ok(unsafe { f(opt.into(), val) } == 1)
    }

    pub fn get_capture_option_u32(
        &self,
        opt: impl Into<sys::RENDERDOC_CaptureOption>,
    ) -> Result<u32, InAppError> {
        let f = self
            .api()
            .GetCaptureOptionU32
            .ok_or(InAppError::MissingFunction("GetCaptureOptionU32"))?;
        Ok(unsafe { f(opt.into()) })
    }

    pub fn get_capture_option_f32(
        &self,
        opt: impl Into<sys::RENDERDOC_CaptureOption>,
    ) -> Result<f32, InAppError> {
        let f = self
            .api()
            .GetCaptureOptionF32
            .ok_or(InAppError::MissingFunction("GetCaptureOptionF32"))?;
        Ok(unsafe { f(opt.into()) })
    }

    pub fn set_focus_toggle_keys<I>(&self, keys: &[I]) -> Result<(), InAppError>
    where
        I: Clone + Into<sys::RENDERDOC_InputButton>,
    {
        let f = self
            .api()
            .SetFocusToggleKeys
            .ok_or(InAppError::MissingFunction("SetFocusToggleKeys"))?;

        if keys.is_empty() {
            unsafe { f(std::ptr::null_mut(), 0) };
            return Ok(());
        }

        let mut owned: Vec<sys::RENDERDOC_InputButton> =
            keys.iter().cloned().map(Into::into).collect();
        let num = i32::try_from(owned.len()).map_err(|_| InAppError::TooManyKeys)?;
        unsafe { f(owned.as_mut_ptr(), num) };
        Ok(())
    }

    pub fn set_capture_keys<I>(&self, keys: &[I]) -> Result<(), InAppError>
    where
        I: Clone + Into<sys::RENDERDOC_InputButton>,
    {
        let f = self
            .api()
            .SetCaptureKeys
            .ok_or(InAppError::MissingFunction("SetCaptureKeys"))?;

        if keys.is_empty() {
            unsafe { f(std::ptr::null_mut(), 0) };
            return Ok(());
        }

        let mut owned: Vec<sys::RENDERDOC_InputButton> =
            keys.iter().cloned().map(Into::into).collect();
        let num = i32::try_from(owned.len()).map_err(|_| InAppError::TooManyKeys)?;
        unsafe { f(owned.as_mut_ptr(), num) };
        Ok(())
    }

    pub fn get_overlay_bits(&self) -> Result<OverlayBits, InAppError> {
        let f = self
            .api()
            .GetOverlayBits
            .ok_or(InAppError::MissingFunction("GetOverlayBits"))?;
        Ok(OverlayBits::from_bits_truncate(unsafe { f() }))
    }

    pub fn mask_overlay_bits(&self, and_mask: u32, or_mask: u32) -> Result<(), InAppError> {
        let f = self
            .api()
            .MaskOverlayBits
            .ok_or(InAppError::MissingFunction("MaskOverlayBits"))?;
        unsafe { f(and_mask, or_mask) };
        Ok(())
    }

    pub fn mask_overlay_bits_flags(
        &self,
        and_mask: OverlayBits,
        or_mask: OverlayBits,
    ) -> Result<(), InAppError> {
        self.mask_overlay_bits(and_mask.bits(), or_mask.bits())
    }

    pub fn is_target_control_connected(&self) -> Result<bool, InAppError> {
        let f = unsafe { self.api().__bindgen_anon_4.IsTargetControlConnected }
            .ok_or(InAppError::MissingFunction("IsTargetControlConnected"))?;
        Ok(unsafe { f() } == 1)
    }

    pub fn launch_replay_ui(
        &self,
        connect_target_control: bool,
        cmdline: Option<&str>,
    ) -> Result<u32, InAppError> {
        let f = self
            .api()
            .LaunchReplayUI
            .ok_or(InAppError::MissingFunction("LaunchReplayUI"))?;
        let cmdline_cstr;
        let cmd_ptr = if let Some(s) = cmdline {
            cmdline_cstr = CString::new(s).map_err(|_| InAppError::InvalidUtf8)?;
            cmdline_cstr.as_ptr()
        } else {
            std::ptr::null()
        };
        let ok = unsafe { f(if connect_target_control { 1 } else { 0 }, cmd_ptr) };
        Ok(ok)
    }

    pub fn show_replay_ui(&self) -> Result<bool, InAppError> {
        let f = self
            .api()
            .ShowReplayUI
            .ok_or(InAppError::MissingFunction("ShowReplayUI"))?;
        Ok(unsafe { f() } == 1)
    }

    pub fn discard_frame_capture(
        &self,
        device: Option<sys::RENDERDOC_DevicePointer>,
        window: Option<sys::RENDERDOC_WindowHandle>,
    ) -> Result<bool, InAppError> {
        let f = self
            .api()
            .DiscardFrameCapture
            .ok_or(InAppError::MissingFunction("DiscardFrameCapture"))?;
        let ok = unsafe {
            f(
                device.unwrap_or(std::ptr::null_mut()),
                window.unwrap_or(std::ptr::null_mut()),
            )
        };
        Ok(ok == 1)
    }

    pub fn set_capture_file_comments(
        &self,
        capture_file_path: Option<&str>,
        comments: &str,
    ) -> Result<(), InAppError> {
        let f = self
            .api()
            .SetCaptureFileComments
            .ok_or(InAppError::MissingFunction("SetCaptureFileComments"))?;
        let comments_c = CString::new(comments).map_err(|_| InAppError::InvalidUtf8)?;
        let path_c;
        let path_ptr = if let Some(p) = capture_file_path {
            path_c = CString::new(p).map_err(|_| InAppError::InvalidUtf8)?;
            path_c.as_ptr()
        } else {
            std::ptr::null()
        };
        unsafe { f(path_ptr, comments_c.as_ptr()) };
        Ok(())
    }

    pub fn set_capture_title(&self, title: &str) -> Result<(), InAppError> {
        let f = self
            .api()
            .SetCaptureTitle
            .ok_or(InAppError::MissingFunction("SetCaptureTitle"))?;
        let title_c = CString::new(title).map_err(|_| InAppError::InvalidUtf8)?;
        unsafe { f(title_c.as_ptr()) };
        Ok(())
    }

    pub fn unload_crash_handler(&self) -> Result<(), InAppError> {
        let f = self
            .api()
            .UnloadCrashHandler
            .ok_or(InAppError::MissingFunction("UnloadCrashHandler"))?;
        unsafe { f() };
        Ok(())
    }

    pub fn remove_hooks(&self) -> Result<(), InAppError> {
        let f = unsafe { self.api().__bindgen_anon_1.RemoveHooks }
            .ok_or(InAppError::MissingFunction("RemoveHooks"))?;
        unsafe { f() };
        Ok(())
    }

    pub fn set_active_window(
        &self,
        device: Option<sys::RENDERDOC_DevicePointer>,
        window: Option<sys::RENDERDOC_WindowHandle>,
    ) -> Result<(), InAppError> {
        let f = self
            .api()
            .SetActiveWindow
            .ok_or(InAppError::MissingFunction("SetActiveWindow"))?;
        unsafe {
            f(
                device.unwrap_or(std::ptr::null_mut()),
                window.unwrap_or(std::ptr::null_mut()),
            )
        };
        Ok(())
    }

    pub fn trigger_capture(&self) -> Result<(), InAppError> {
        let f = self
            .api()
            .TriggerCapture
            .ok_or(InAppError::MissingFunction("TriggerCapture"))?;
        unsafe { f() };
        Ok(())
    }

    pub fn trigger_multi_frame_capture(&self, frames: u32) -> Result<(), InAppError> {
        let f = self
            .api()
            .TriggerMultiFrameCapture
            .ok_or(InAppError::MissingFunction("TriggerMultiFrameCapture"))?;
        unsafe { f(frames) };
        Ok(())
    }

    pub fn start_frame_capture(
        &self,
        device: Option<sys::RENDERDOC_DevicePointer>,
        window: Option<sys::RENDERDOC_WindowHandle>,
    ) -> Result<(), InAppError> {
        let f = self
            .api()
            .StartFrameCapture
            .ok_or(InAppError::MissingFunction("StartFrameCapture"))?;
        unsafe {
            f(
                device.unwrap_or(std::ptr::null_mut()),
                window.unwrap_or(std::ptr::null_mut()),
            )
        };
        Ok(())
    }

    pub fn end_frame_capture(
        &self,
        device: Option<sys::RENDERDOC_DevicePointer>,
        window: Option<sys::RENDERDOC_WindowHandle>,
    ) -> Result<bool, InAppError> {
        let f = self
            .api()
            .EndFrameCapture
            .ok_or(InAppError::MissingFunction("EndFrameCapture"))?;
        let ok = unsafe {
            f(
                device.unwrap_or(std::ptr::null_mut()),
                window.unwrap_or(std::ptr::null_mut()),
            )
        };
        Ok(ok == 1)
    }

    pub fn is_frame_capturing(&self) -> Result<bool, InAppError> {
        let f = self
            .api()
            .IsFrameCapturing
            .ok_or(InAppError::MissingFunction("IsFrameCapturing"))?;
        Ok(unsafe { f() } == 1)
    }

    pub fn get_num_captures(&self) -> Result<u32, InAppError> {
        let f = self
            .api()
            .GetNumCaptures
            .ok_or(InAppError::MissingFunction("GetNumCaptures"))?;
        Ok(unsafe { f() })
    }

    pub fn get_capture(&self, idx: u32) -> Result<(String, u64), InAppError> {
        let f = self
            .api()
            .GetCapture
            .ok_or(InAppError::MissingFunction("GetCapture"))?;

        let mut path_len: u32 = 0;
        let mut timestamp: u64 = 0;
        let ok = unsafe { f(idx, std::ptr::null_mut(), &mut path_len, &mut timestamp) };
        if ok == 0 {
            return Err(InAppError::InvalidCaptureIndex);
        }

        let mut buf = vec![0u8; path_len as usize];
        let ok = unsafe {
            f(
                idx,
                buf.as_mut_ptr() as *mut i8,
                &mut path_len,
                &mut timestamp,
            )
        };
        if ok == 0 {
            return Err(InAppError::InvalidCaptureIndex);
        }

        while buf.last().copied() == Some(0) {
            buf.pop();
        }
        let s = std::str::from_utf8(&buf).map_err(|_| InAppError::InvalidUtf8)?;

        Ok((s.to_string(), timestamp))
    }

    pub fn get_capture_info(&self, idx: u32) -> Result<(PathBuf, SystemTime), InAppError> {
        let (path, timestamp_s) = self.get_capture(idx)?;
        Ok((
            PathBuf::from(path),
            UNIX_EPOCH + Duration::from_secs(timestamp_s),
        ))
    }

    pub fn get_capture_info_opt(
        &self,
        idx: u32,
    ) -> Result<Option<(PathBuf, SystemTime)>, InAppError> {
        match self.get_capture_info(idx) {
            Ok(v) => Ok(Some(v)),
            Err(InAppError::InvalidCaptureIndex) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
