/* pregenerated minimal bindings for renderdoc_app.h (API 1.6.0) */

use core::ffi::{c_char, c_float, c_int, c_uint, c_void};

pub type RENDERDOC_DevicePointer = *mut c_void;
pub type RENDERDOC_WindowHandle = *mut c_void;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RENDERDOC_InputButton {
    // '0' - '9' matches ASCII values
    eRENDERDOC_Key_0 = 0x30,
    eRENDERDOC_Key_1 = 0x31,
    eRENDERDOC_Key_2 = 0x32,
    eRENDERDOC_Key_3 = 0x33,
    eRENDERDOC_Key_4 = 0x34,
    eRENDERDOC_Key_5 = 0x35,
    eRENDERDOC_Key_6 = 0x36,
    eRENDERDOC_Key_7 = 0x37,
    eRENDERDOC_Key_8 = 0x38,
    eRENDERDOC_Key_9 = 0x39,

    // 'A' - 'Z' matches ASCII values
    eRENDERDOC_Key_A = 0x41,
    eRENDERDOC_Key_B = 0x42,
    eRENDERDOC_Key_C = 0x43,
    eRENDERDOC_Key_D = 0x44,
    eRENDERDOC_Key_E = 0x45,
    eRENDERDOC_Key_F = 0x46,
    eRENDERDOC_Key_G = 0x47,
    eRENDERDOC_Key_H = 0x48,
    eRENDERDOC_Key_I = 0x49,
    eRENDERDOC_Key_J = 0x4A,
    eRENDERDOC_Key_K = 0x4B,
    eRENDERDOC_Key_L = 0x4C,
    eRENDERDOC_Key_M = 0x4D,
    eRENDERDOC_Key_N = 0x4E,
    eRENDERDOC_Key_O = 0x4F,
    eRENDERDOC_Key_P = 0x50,
    eRENDERDOC_Key_Q = 0x51,
    eRENDERDOC_Key_R = 0x52,
    eRENDERDOC_Key_S = 0x53,
    eRENDERDOC_Key_T = 0x54,
    eRENDERDOC_Key_U = 0x55,
    eRENDERDOC_Key_V = 0x56,
    eRENDERDOC_Key_W = 0x57,
    eRENDERDOC_Key_X = 0x58,
    eRENDERDOC_Key_Y = 0x59,
    eRENDERDOC_Key_Z = 0x5A,

    // leave the rest of the ASCII range free
    // in case we want to use it later
    eRENDERDOC_Key_NonPrintable = 0x100,

    eRENDERDOC_Key_Divide,
    eRENDERDOC_Key_Multiply,
    eRENDERDOC_Key_Subtract,
    eRENDERDOC_Key_Plus,

    eRENDERDOC_Key_F1,
    eRENDERDOC_Key_F2,
    eRENDERDOC_Key_F3,
    eRENDERDOC_Key_F4,
    eRENDERDOC_Key_F5,
    eRENDERDOC_Key_F6,
    eRENDERDOC_Key_F7,
    eRENDERDOC_Key_F8,
    eRENDERDOC_Key_F9,
    eRENDERDOC_Key_F10,
    eRENDERDOC_Key_F11,
    eRENDERDOC_Key_F12,

    eRENDERDOC_Key_Home,
    eRENDERDOC_Key_End,
    eRENDERDOC_Key_Insert,
    eRENDERDOC_Key_Delete,
    eRENDERDOC_Key_PageUp,
    eRENDERDOC_Key_PageDn,

    eRENDERDOC_Key_Backspace,
    eRENDERDOC_Key_Tab,
    eRENDERDOC_Key_PrtScrn,
    eRENDERDOC_Key_Pause,

    eRENDERDOC_Key_Max,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RENDERDOC_OverlayBits {
    eRENDERDOC_Overlay_Enabled = 0x1,
    eRENDERDOC_Overlay_FrameRate = 0x2,
    eRENDERDOC_Overlay_FrameNumber = 0x4,
    eRENDERDOC_Overlay_CaptureList = 0x8,
    eRENDERDOC_Overlay_Default = 0x1 | 0x2 | 0x4 | 0x8,
    eRENDERDOC_Overlay_All = 0x7ffffff,
    eRENDERDOC_Overlay_None = 0,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RENDERDOC_Version {
    eRENDERDOC_API_Version_1_0_0 = 10000,
    eRENDERDOC_API_Version_1_0_1 = 10001,
    eRENDERDOC_API_Version_1_0_2 = 10002,
    eRENDERDOC_API_Version_1_1_0 = 10100,
    eRENDERDOC_API_Version_1_1_1 = 10101,
    eRENDERDOC_API_Version_1_1_2 = 10102,
    eRENDERDOC_API_Version_1_2_0 = 10200,
    eRENDERDOC_API_Version_1_3_0 = 10300,
    eRENDERDOC_API_Version_1_4_0 = 10400,
    eRENDERDOC_API_Version_1_4_1 = 10401,
    eRENDERDOC_API_Version_1_4_2 = 10402,
    eRENDERDOC_API_Version_1_5_0 = 10500,
    eRENDERDOC_API_Version_1_6_0 = 10600,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RENDERDOC_CaptureOption {
    eRENDERDOC_Option_AllowVSync = 0,
    eRENDERDOC_Option_AllowFullscreen = 1,
    eRENDERDOC_Option_APIValidation = 2,
    eRENDERDOC_Option_CaptureCallstacks = 3,
    eRENDERDOC_Option_CaptureCallstacksOnlyDraws = 4,
    eRENDERDOC_Option_DelayForDebugger = 5,
    eRENDERDOC_Option_VerifyBufferAccess = 6,
    eRENDERDOC_Option_HookIntoChildren = 7,
    eRENDERDOC_Option_RefAllResources = 8,
    eRENDERDOC_Option_SaveAllInitials = 9,
    eRENDERDOC_Option_CaptureAllCmdLists = 10,
    eRENDERDOC_Option_DebugOutputMute = 11,
    eRENDERDOC_Option_AllowUnsupportedVendorExtensions = 12,
    eRENDERDOC_Option_SoftMemoryLimit = 13,
}

pub type pRENDERDOC_SetCaptureOptionU32 =
    unsafe extern "C" fn(opt: RENDERDOC_CaptureOption, val: c_uint) -> c_int;
pub type pRENDERDOC_SetCaptureOptionF32 =
    unsafe extern "C" fn(opt: RENDERDOC_CaptureOption, val: c_float) -> c_int;
pub type pRENDERDOC_GetCaptureOptionU32 = unsafe extern "C" fn(opt: RENDERDOC_CaptureOption) -> c_uint;
pub type pRENDERDOC_GetCaptureOptionF32 =
    unsafe extern "C" fn(opt: RENDERDOC_CaptureOption) -> c_float;

pub type pRENDERDOC_SetFocusToggleKeys =
    unsafe extern "C" fn(keys: *mut RENDERDOC_InputButton, num: c_int);
pub type pRENDERDOC_SetCaptureKeys =
    unsafe extern "C" fn(keys: *mut RENDERDOC_InputButton, num: c_int);

pub type pRENDERDOC_GetOverlayBits = unsafe extern "C" fn() -> c_uint;
pub type pRENDERDOC_MaskOverlayBits = unsafe extern "C" fn(and_mask: c_uint, or_mask: c_uint);

pub type pRENDERDOC_RemoveHooks = unsafe extern "C" fn();
pub type pRENDERDOC_Shutdown = pRENDERDOC_RemoveHooks;
pub type pRENDERDOC_UnloadCrashHandler = unsafe extern "C" fn();

pub type pRENDERDOC_SetCaptureFilePathTemplate = unsafe extern "C" fn(pathtemplate: *const c_char);
pub type pRENDERDOC_GetCaptureFilePathTemplate = unsafe extern "C" fn() -> *const c_char;

pub type pRENDERDOC_SetLogFilePathTemplate = pRENDERDOC_SetCaptureFilePathTemplate;
pub type pRENDERDOC_GetLogFilePathTemplate = pRENDERDOC_GetCaptureFilePathTemplate;

pub type pRENDERDOC_GetNumCaptures = unsafe extern "C" fn() -> c_uint;
pub type pRENDERDOC_GetCapture = unsafe extern "C" fn(
    idx: c_uint,
    filename: *mut c_char,
    pathlength: *mut c_uint,
    timestamp: *mut u64,
) -> c_uint;

pub type pRENDERDOC_TriggerCapture = unsafe extern "C" fn();
pub type pRENDERDOC_TriggerMultiFrameCapture = unsafe extern "C" fn(numFrames: c_uint);

pub type pRENDERDOC_IsTargetControlConnected = unsafe extern "C" fn() -> c_uint;
pub type pRENDERDOC_IsRemoteAccessConnected = pRENDERDOC_IsTargetControlConnected;

pub type pRENDERDOC_SetActiveWindow =
    unsafe extern "C" fn(device: RENDERDOC_DevicePointer, wndHandle: RENDERDOC_WindowHandle);

pub type pRENDERDOC_StartFrameCapture =
    unsafe extern "C" fn(device: RENDERDOC_DevicePointer, wndHandle: RENDERDOC_WindowHandle);
pub type pRENDERDOC_IsFrameCapturing = unsafe extern "C" fn() -> c_uint;
pub type pRENDERDOC_EndFrameCapture =
    unsafe extern "C" fn(device: RENDERDOC_DevicePointer, wndHandle: RENDERDOC_WindowHandle) -> c_uint;
pub type pRENDERDOC_DiscardFrameCapture =
    unsafe extern "C" fn(device: RENDERDOC_DevicePointer, wndHandle: RENDERDOC_WindowHandle) -> c_uint;

pub type pRENDERDOC_GetAPIVersion = unsafe extern "C" fn(major: *mut c_int, minor: *mut c_int, patch: *mut c_int);
pub type pRENDERDOC_ShowReplayUI = unsafe extern "C" fn() -> c_uint;
pub type pRENDERDOC_LaunchReplayUI = unsafe extern "C" fn(connectTargetControl: c_uint, cmdline: *const c_char) -> c_uint;

pub type pRENDERDOC_SetCaptureFileComments = unsafe extern "C" fn(filePath: *const c_char, comments: *const c_char);
pub type pRENDERDOC_SetCaptureTitle = unsafe extern "C" fn(title: *const c_char);

pub type pRENDERDOC_GetAPI = unsafe extern "C" fn(version: RENDERDOC_Version, outAPIPointers: *mut *mut c_void) -> c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub union RENDERDOC_API_1_6_0__bindgen_union_1 {
    pub Shutdown: Option<pRENDERDOC_Shutdown>,
    pub RemoveHooks: Option<pRENDERDOC_RemoveHooks>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RENDERDOC_API_1_6_0__bindgen_union_2 {
    pub SetLogFilePathTemplate: Option<pRENDERDOC_SetLogFilePathTemplate>,
    pub SetCaptureFilePathTemplate: Option<pRENDERDOC_SetCaptureFilePathTemplate>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RENDERDOC_API_1_6_0__bindgen_union_3 {
    pub GetLogFilePathTemplate: Option<pRENDERDOC_GetLogFilePathTemplate>,
    pub GetCaptureFilePathTemplate: Option<pRENDERDOC_GetCaptureFilePathTemplate>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RENDERDOC_API_1_6_0__bindgen_union_4 {
    pub IsRemoteAccessConnected: Option<pRENDERDOC_IsRemoteAccessConnected>,
    pub IsTargetControlConnected: Option<pRENDERDOC_IsTargetControlConnected>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RENDERDOC_API_1_6_0 {
    pub GetAPIVersion: Option<pRENDERDOC_GetAPIVersion>,
    pub SetCaptureOptionU32: Option<pRENDERDOC_SetCaptureOptionU32>,
    pub SetCaptureOptionF32: Option<pRENDERDOC_SetCaptureOptionF32>,
    pub GetCaptureOptionU32: Option<pRENDERDOC_GetCaptureOptionU32>,
    pub GetCaptureOptionF32: Option<pRENDERDOC_GetCaptureOptionF32>,
    pub SetFocusToggleKeys: Option<pRENDERDOC_SetFocusToggleKeys>,
    pub SetCaptureKeys: Option<pRENDERDOC_SetCaptureKeys>,
    pub GetOverlayBits: Option<pRENDERDOC_GetOverlayBits>,
    pub MaskOverlayBits: Option<pRENDERDOC_MaskOverlayBits>,
    pub __bindgen_anon_1: RENDERDOC_API_1_6_0__bindgen_union_1,
    pub UnloadCrashHandler: Option<pRENDERDOC_UnloadCrashHandler>,
    pub __bindgen_anon_2: RENDERDOC_API_1_6_0__bindgen_union_2,
    pub __bindgen_anon_3: RENDERDOC_API_1_6_0__bindgen_union_3,
    pub GetNumCaptures: Option<pRENDERDOC_GetNumCaptures>,
    pub GetCapture: Option<pRENDERDOC_GetCapture>,
    pub TriggerCapture: Option<pRENDERDOC_TriggerCapture>,
    pub __bindgen_anon_4: RENDERDOC_API_1_6_0__bindgen_union_4,
    pub LaunchReplayUI: Option<pRENDERDOC_LaunchReplayUI>,
    pub SetActiveWindow: Option<pRENDERDOC_SetActiveWindow>,
    pub StartFrameCapture: Option<pRENDERDOC_StartFrameCapture>,
    pub IsFrameCapturing: Option<pRENDERDOC_IsFrameCapturing>,
    pub EndFrameCapture: Option<pRENDERDOC_EndFrameCapture>,
    pub TriggerMultiFrameCapture: Option<pRENDERDOC_TriggerMultiFrameCapture>,
    pub SetCaptureFileComments: Option<pRENDERDOC_SetCaptureFileComments>,
    pub DiscardFrameCapture: Option<pRENDERDOC_DiscardFrameCapture>,
    pub ShowReplayUI: Option<pRENDERDOC_ShowReplayUI>,
    pub SetCaptureTitle: Option<pRENDERDOC_SetCaptureTitle>,
}

pub type RENDERDOC_API_1_0_0 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_0_1 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_0_2 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_1_0 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_1_1 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_1_2 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_2_0 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_3_0 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_4_0 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_4_1 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_4_2 = RENDERDOC_API_1_6_0;
pub type RENDERDOC_API_1_5_0 = RENDERDOC_API_1_6_0;
