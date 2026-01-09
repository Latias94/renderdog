mod in_app;
mod renderdog;
mod settings;

pub use in_app::*;
pub use renderdog::*;
pub use settings::*;

pub type SysCaptureOption = RENDERDOC_CaptureOption;
pub type SysInputButton = RENDERDOC_InputButton;
pub type SysDevicePointer = RENDERDOC_DevicePointer;
pub type SysWindowHandle = RENDERDOC_WindowHandle;
