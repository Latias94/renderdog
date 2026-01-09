use bitflags::bitflags;

use renderdog_sys as sys;

/// RenderDoc capture options (strongly typed wrapper).
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CaptureOption {
    AllowVSync,
    AllowFullscreen,
    ApiValidation,
    CaptureCallstacks,
    CaptureCallstacksOnlyDraws,
    DelayForDebugger,
    VerifyBufferAccess,
    HookIntoChildren,
    RefAllResources,
    SaveAllInitials,
    CaptureAllCmdLists,
    DebugOutputMute,
    AllowUnsupportedVendorExtensions,
    SoftMemoryLimit,
}

impl From<CaptureOption> for sys::RENDERDOC_CaptureOption {
    fn from(value: CaptureOption) -> Self {
        match value {
            CaptureOption::AllowVSync => sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_AllowVSync,
            CaptureOption::AllowFullscreen => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_AllowFullscreen
            }
            CaptureOption::ApiValidation => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_APIValidation
            }
            CaptureOption::CaptureCallstacks => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_CaptureCallstacks
            }
            CaptureOption::CaptureCallstacksOnlyDraws => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_CaptureCallstacksOnlyDraws
            }
            CaptureOption::DelayForDebugger => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_DelayForDebugger
            }
            CaptureOption::VerifyBufferAccess => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_VerifyBufferAccess
            }
            CaptureOption::HookIntoChildren => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_HookIntoChildren
            }
            CaptureOption::RefAllResources => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_RefAllResources
            }
            CaptureOption::SaveAllInitials => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_SaveAllInitials
            }
            CaptureOption::CaptureAllCmdLists => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_CaptureAllCmdLists
            }
            CaptureOption::DebugOutputMute => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_DebugOutputMute
            }
            CaptureOption::AllowUnsupportedVendorExtensions => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_AllowUnsupportedVendorExtensions
            }
            CaptureOption::SoftMemoryLimit => {
                sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_SoftMemoryLimit
            }
        }
    }
}

/// RenderDoc input buttons (strongly typed wrapper).
#[allow(missing_docs)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InputButton {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Divide,
    Multiply,
    Subtract,
    Plus,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDn,
    Backspace,
    Tab,
    PrtScrn,
    Pause,
    Max,
}

impl From<InputButton> for sys::RENDERDOC_InputButton {
    fn from(value: InputButton) -> Self {
        match value {
            InputButton::Key0 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_0,
            InputButton::Key1 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_1,
            InputButton::Key2 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_2,
            InputButton::Key3 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_3,
            InputButton::Key4 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_4,
            InputButton::Key5 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_5,
            InputButton::Key6 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_6,
            InputButton::Key7 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_7,
            InputButton::Key8 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_8,
            InputButton::Key9 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_9,
            InputButton::A => sys::RENDERDOC_InputButton::eRENDERDOC_Key_A,
            InputButton::B => sys::RENDERDOC_InputButton::eRENDERDOC_Key_B,
            InputButton::C => sys::RENDERDOC_InputButton::eRENDERDOC_Key_C,
            InputButton::D => sys::RENDERDOC_InputButton::eRENDERDOC_Key_D,
            InputButton::E => sys::RENDERDOC_InputButton::eRENDERDOC_Key_E,
            InputButton::F => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F,
            InputButton::G => sys::RENDERDOC_InputButton::eRENDERDOC_Key_G,
            InputButton::H => sys::RENDERDOC_InputButton::eRENDERDOC_Key_H,
            InputButton::I => sys::RENDERDOC_InputButton::eRENDERDOC_Key_I,
            InputButton::J => sys::RENDERDOC_InputButton::eRENDERDOC_Key_J,
            InputButton::K => sys::RENDERDOC_InputButton::eRENDERDOC_Key_K,
            InputButton::L => sys::RENDERDOC_InputButton::eRENDERDOC_Key_L,
            InputButton::M => sys::RENDERDOC_InputButton::eRENDERDOC_Key_M,
            InputButton::N => sys::RENDERDOC_InputButton::eRENDERDOC_Key_N,
            InputButton::O => sys::RENDERDOC_InputButton::eRENDERDOC_Key_O,
            InputButton::P => sys::RENDERDOC_InputButton::eRENDERDOC_Key_P,
            InputButton::Q => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Q,
            InputButton::R => sys::RENDERDOC_InputButton::eRENDERDOC_Key_R,
            InputButton::S => sys::RENDERDOC_InputButton::eRENDERDOC_Key_S,
            InputButton::T => sys::RENDERDOC_InputButton::eRENDERDOC_Key_T,
            InputButton::U => sys::RENDERDOC_InputButton::eRENDERDOC_Key_U,
            InputButton::V => sys::RENDERDOC_InputButton::eRENDERDOC_Key_V,
            InputButton::W => sys::RENDERDOC_InputButton::eRENDERDOC_Key_W,
            InputButton::X => sys::RENDERDOC_InputButton::eRENDERDOC_Key_X,
            InputButton::Y => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Y,
            InputButton::Z => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Z,
            InputButton::Divide => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Divide,
            InputButton::Multiply => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Multiply,
            InputButton::Subtract => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Subtract,
            InputButton::Plus => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Plus,
            InputButton::F1 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F1,
            InputButton::F2 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F2,
            InputButton::F3 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F3,
            InputButton::F4 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F4,
            InputButton::F5 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F5,
            InputButton::F6 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F6,
            InputButton::F7 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F7,
            InputButton::F8 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F8,
            InputButton::F9 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F9,
            InputButton::F10 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F10,
            InputButton::F11 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F11,
            InputButton::F12 => sys::RENDERDOC_InputButton::eRENDERDOC_Key_F12,
            InputButton::Home => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Home,
            InputButton::End => sys::RENDERDOC_InputButton::eRENDERDOC_Key_End,
            InputButton::Insert => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Insert,
            InputButton::Delete => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Delete,
            InputButton::PageUp => sys::RENDERDOC_InputButton::eRENDERDOC_Key_PageUp,
            InputButton::PageDn => sys::RENDERDOC_InputButton::eRENDERDOC_Key_PageDn,
            InputButton::Backspace => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Backspace,
            InputButton::Tab => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Tab,
            InputButton::PrtScrn => sys::RENDERDOC_InputButton::eRENDERDOC_Key_PrtScrn,
            InputButton::Pause => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Pause,
            InputButton::Max => sys::RENDERDOC_InputButton::eRENDERDOC_Key_Max,
        }
    }
}

bitflags! {
    pub struct OverlayBits: u32 {
        const ENABLED = sys::RENDERDOC_OverlayBits::eRENDERDOC_Overlay_Enabled as u32;
        const FRAME_RATE = sys::RENDERDOC_OverlayBits::eRENDERDOC_Overlay_FrameRate as u32;
        const FRAME_NUMBER = sys::RENDERDOC_OverlayBits::eRENDERDOC_Overlay_FrameNumber as u32;
        const CAPTURE_LIST = sys::RENDERDOC_OverlayBits::eRENDERDOC_Overlay_CaptureList as u32;
        const DEFAULT = sys::RENDERDOC_OverlayBits::eRENDERDOC_Overlay_Default as u32;
        const ALL = sys::RENDERDOC_OverlayBits::eRENDERDOC_Overlay_All as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_option_maps_to_sys() {
        let sys_opt: sys::RENDERDOC_CaptureOption = CaptureOption::DelayForDebugger.into();
        assert_eq!(
            sys_opt,
            sys::RENDERDOC_CaptureOption::eRENDERDOC_Option_DelayForDebugger
        );
    }

    #[test]
    fn input_button_maps_to_sys() {
        let sys_btn: sys::RENDERDOC_InputButton = InputButton::F12.into();
        assert_eq!(sys_btn, sys::RENDERDOC_InputButton::eRENDERDOC_Key_F12);
    }
}
