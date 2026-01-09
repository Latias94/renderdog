//! Winit integration helpers for `renderdog` (in-app RenderDoc API).
//!
//! This crate is intentionally small: it provides convenience conversions for key codes and
//! (on Windows) extracting a native window handle for RenderDoc APIs that accept a window handle.

use renderdog::RENDERDOC_InputButton;

/// Convert a `winit` key code into a RenderDoc input button.
///
/// Unhandled keys map to `eRENDERDOC_Key_Max`.
pub fn input_button_from_key_code(code: winit::keyboard::KeyCode) -> RENDERDOC_InputButton {
    use winit::keyboard::KeyCode;
    match code {
        KeyCode::Digit1 => RENDERDOC_InputButton::eRENDERDOC_Key_1,
        KeyCode::Digit2 => RENDERDOC_InputButton::eRENDERDOC_Key_2,
        KeyCode::Digit3 => RENDERDOC_InputButton::eRENDERDOC_Key_3,
        KeyCode::Digit4 => RENDERDOC_InputButton::eRENDERDOC_Key_4,
        KeyCode::Digit5 => RENDERDOC_InputButton::eRENDERDOC_Key_5,
        KeyCode::Digit6 => RENDERDOC_InputButton::eRENDERDOC_Key_6,
        KeyCode::Digit7 => RENDERDOC_InputButton::eRENDERDOC_Key_7,
        KeyCode::Digit8 => RENDERDOC_InputButton::eRENDERDOC_Key_8,
        KeyCode::Digit9 => RENDERDOC_InputButton::eRENDERDOC_Key_9,
        KeyCode::Digit0 => RENDERDOC_InputButton::eRENDERDOC_Key_0,

        KeyCode::KeyA => RENDERDOC_InputButton::eRENDERDOC_Key_A,
        KeyCode::KeyB => RENDERDOC_InputButton::eRENDERDOC_Key_B,
        KeyCode::KeyC => RENDERDOC_InputButton::eRENDERDOC_Key_C,
        KeyCode::KeyD => RENDERDOC_InputButton::eRENDERDOC_Key_D,
        KeyCode::KeyE => RENDERDOC_InputButton::eRENDERDOC_Key_E,
        KeyCode::KeyF => RENDERDOC_InputButton::eRENDERDOC_Key_F,
        KeyCode::KeyG => RENDERDOC_InputButton::eRENDERDOC_Key_G,
        KeyCode::KeyH => RENDERDOC_InputButton::eRENDERDOC_Key_H,
        KeyCode::KeyI => RENDERDOC_InputButton::eRENDERDOC_Key_I,
        KeyCode::KeyJ => RENDERDOC_InputButton::eRENDERDOC_Key_J,
        KeyCode::KeyK => RENDERDOC_InputButton::eRENDERDOC_Key_K,
        KeyCode::KeyL => RENDERDOC_InputButton::eRENDERDOC_Key_L,
        KeyCode::KeyM => RENDERDOC_InputButton::eRENDERDOC_Key_M,
        KeyCode::KeyN => RENDERDOC_InputButton::eRENDERDOC_Key_N,
        KeyCode::KeyO => RENDERDOC_InputButton::eRENDERDOC_Key_O,
        KeyCode::KeyP => RENDERDOC_InputButton::eRENDERDOC_Key_P,
        KeyCode::KeyQ => RENDERDOC_InputButton::eRENDERDOC_Key_Q,
        KeyCode::KeyR => RENDERDOC_InputButton::eRENDERDOC_Key_R,
        KeyCode::KeyS => RENDERDOC_InputButton::eRENDERDOC_Key_S,
        KeyCode::KeyT => RENDERDOC_InputButton::eRENDERDOC_Key_T,
        KeyCode::KeyU => RENDERDOC_InputButton::eRENDERDOC_Key_U,
        KeyCode::KeyV => RENDERDOC_InputButton::eRENDERDOC_Key_V,
        KeyCode::KeyW => RENDERDOC_InputButton::eRENDERDOC_Key_W,
        KeyCode::KeyX => RENDERDOC_InputButton::eRENDERDOC_Key_X,
        KeyCode::KeyY => RENDERDOC_InputButton::eRENDERDOC_Key_Y,
        KeyCode::KeyZ => RENDERDOC_InputButton::eRENDERDOC_Key_Z,

        KeyCode::NumpadDivide => RENDERDOC_InputButton::eRENDERDOC_Key_Divide,
        KeyCode::NumpadMultiply => RENDERDOC_InputButton::eRENDERDOC_Key_Multiply,
        KeyCode::NumpadSubtract => RENDERDOC_InputButton::eRENDERDOC_Key_Subtract,
        KeyCode::NumpadAdd => RENDERDOC_InputButton::eRENDERDOC_Key_Plus,

        KeyCode::F1 => RENDERDOC_InputButton::eRENDERDOC_Key_F1,
        KeyCode::F2 => RENDERDOC_InputButton::eRENDERDOC_Key_F2,
        KeyCode::F3 => RENDERDOC_InputButton::eRENDERDOC_Key_F3,
        KeyCode::F4 => RENDERDOC_InputButton::eRENDERDOC_Key_F4,
        KeyCode::F5 => RENDERDOC_InputButton::eRENDERDOC_Key_F5,
        KeyCode::F6 => RENDERDOC_InputButton::eRENDERDOC_Key_F6,
        KeyCode::F7 => RENDERDOC_InputButton::eRENDERDOC_Key_F7,
        KeyCode::F8 => RENDERDOC_InputButton::eRENDERDOC_Key_F8,
        KeyCode::F9 => RENDERDOC_InputButton::eRENDERDOC_Key_F9,
        KeyCode::F10 => RENDERDOC_InputButton::eRENDERDOC_Key_F10,
        KeyCode::F11 => RENDERDOC_InputButton::eRENDERDOC_Key_F11,
        KeyCode::F12 => RENDERDOC_InputButton::eRENDERDOC_Key_F12,

        KeyCode::Home => RENDERDOC_InputButton::eRENDERDOC_Key_Home,
        KeyCode::End => RENDERDOC_InputButton::eRENDERDOC_Key_End,
        KeyCode::Insert => RENDERDOC_InputButton::eRENDERDOC_Key_Insert,
        KeyCode::Delete => RENDERDOC_InputButton::eRENDERDOC_Key_Delete,
        KeyCode::PageUp => RENDERDOC_InputButton::eRENDERDOC_Key_PageUp,
        KeyCode::PageDown => RENDERDOC_InputButton::eRENDERDOC_Key_PageDn,

        KeyCode::Backspace => RENDERDOC_InputButton::eRENDERDOC_Key_Backspace,
        KeyCode::Tab => RENDERDOC_InputButton::eRENDERDOC_Key_Tab,
        KeyCode::PrintScreen => RENDERDOC_InputButton::eRENDERDOC_Key_PrtScrn,
        KeyCode::Pause => RENDERDOC_InputButton::eRENDERDOC_Key_Pause,

        _ => RENDERDOC_InputButton::eRENDERDOC_Key_Max,
    }
}

/// Extract a native window handle for RenderDoc from a winit window (Windows only).
///
/// Returns `None` on unsupported platforms or when the handle is not available.
#[cfg(windows)]
pub fn renderdoc_window_handle(
    window: &winit::window::Window,
) -> Option<renderdog::RENDERDOC_WindowHandle> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as renderdog::RENDERDOC_WindowHandle),
        _ => None,
    }
}

#[cfg(not(windows))]
pub fn renderdoc_window_handle(
    _window: &winit::window::Window,
) -> Option<renderdog::RENDERDOC_WindowHandle> {
    None
}

/// Start a RenderDoc frame capture using a winit window handle (no device pointer).
#[cfg(windows)]
pub fn start_frame_capture_window(
    rd: &renderdog::RenderDocInApp,
    window: &winit::window::Window,
) -> Result<(), renderdog::InAppError> {
    rd.start_frame_capture(None, renderdoc_window_handle(window))
}

/// End a RenderDoc frame capture using a winit window handle (no device pointer).
#[cfg(windows)]
pub fn end_frame_capture_window(
    rd: &renderdog::RenderDocInApp,
    window: &winit::window::Window,
) -> Result<bool, renderdog::InAppError> {
    rd.end_frame_capture(None, renderdoc_window_handle(window))
}

/// Discard a RenderDoc frame capture using a winit window handle (no device pointer).
#[cfg(windows)]
pub fn discard_frame_capture_window(
    rd: &renderdog::RenderDocInApp,
    window: &winit::window::Window,
) -> Result<bool, renderdog::InAppError> {
    rd.discard_frame_capture(None, renderdoc_window_handle(window))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mapping_basic() {
        use winit::keyboard::KeyCode as K;

        assert_eq!(
            input_button_from_key_code(K::F12),
            RENDERDOC_InputButton::eRENDERDOC_Key_F12
        );
        assert_eq!(
            input_button_from_key_code(K::PrintScreen),
            RENDERDOC_InputButton::eRENDERDOC_Key_PrtScrn
        );
        assert_eq!(
            input_button_from_key_code(K::NumpadDivide),
            RENDERDOC_InputButton::eRENDERDOC_Key_Divide
        );
        assert_eq!(
            input_button_from_key_code(K::KeyA),
            RENDERDOC_InputButton::eRENDERDOC_Key_A
        );
    }
}
