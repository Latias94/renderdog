/*
 * Minimal subset header for RenderDoc in-application API (renderdoc_app.h).
 *
 * This header exists only to support optional bindgen regeneration when
 * RENDERDOG_SYS_REGEN_BINDINGS=1 and the `bindgen` feature is enabled.
 *
 * For the authoritative API, see: https://renderdoc.org/docs/in_application_api.html
 */

#pragma once

#include <stdint.h>

#if defined(_WIN32)
#define RENDERDOC_CC __cdecl
#else
#define RENDERDOC_CC
#endif

typedef void *RENDERDOC_DevicePointer;
typedef void *RENDERDOC_WindowHandle;

typedef enum RENDERDOC_CaptureOption
{
  eRENDERDOC_Option_AllowVSync = 0,
  eRENDERDOC_Option_AllowFullscreen = 1,
  eRENDERDOC_Option_APIValidation = 2,
  eRENDERDOC_Option_CaptureCallstacks = 3,
  eRENDERDOC_Option_CaptureCallstacksOnlyActions = 4,
  eRENDERDOC_Option_DelayForDebugger = 5,
  eRENDERDOC_Option_VerifyBufferAccess = 6,
  eRENDERDOC_Option_HookIntoChildren = 7,
  eRENDERDOC_Option_RefAllResources = 8,
  eRENDERDOC_Option_SaveAllInitials = 9,
  eRENDERDOC_Option_CaptureAllCmdLists = 10,
  eRENDERDOC_Option_DebugOutputMute = 11,
  eRENDERDOC_Option_AllowUnsupportedVendorExtensions = 12,
  eRENDERDOC_Option_SoftMemoryLimit = 13,
} RENDERDOC_CaptureOption;

typedef int(RENDERDOC_CC *pRENDERDOC_SetCaptureOptionU32)(RENDERDOC_CaptureOption opt, uint32_t val);
typedef int(RENDERDOC_CC *pRENDERDOC_SetCaptureOptionF32)(RENDERDOC_CaptureOption opt, float val);
typedef uint32_t(RENDERDOC_CC *pRENDERDOC_GetCaptureOptionU32)(RENDERDOC_CaptureOption opt);
typedef float(RENDERDOC_CC *pRENDERDOC_GetCaptureOptionF32)(RENDERDOC_CaptureOption opt);

typedef enum RENDERDOC_InputButton
{
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
} RENDERDOC_InputButton;

typedef void(RENDERDOC_CC *pRENDERDOC_SetFocusToggleKeys)(RENDERDOC_InputButton *keys, int num);
typedef void(RENDERDOC_CC *pRENDERDOC_SetCaptureKeys)(RENDERDOC_InputButton *keys, int num);

typedef uint32_t(RENDERDOC_CC *pRENDERDOC_GetOverlayBits)(void);
typedef void(RENDERDOC_CC *pRENDERDOC_MaskOverlayBits)(uint32_t And, uint32_t Or);

typedef void(RENDERDOC_CC *pRENDERDOC_RemoveHooks)(void);
typedef pRENDERDOC_RemoveHooks pRENDERDOC_Shutdown;
typedef void(RENDERDOC_CC *pRENDERDOC_UnloadCrashHandler)(void);

typedef void(RENDERDOC_CC *pRENDERDOC_SetCaptureFilePathTemplate)(const char *pathtemplate);
typedef const char *(RENDERDOC_CC *pRENDERDOC_GetCaptureFilePathTemplate)(void);
typedef pRENDERDOC_SetCaptureFilePathTemplate pRENDERDOC_SetLogFilePathTemplate;
typedef pRENDERDOC_GetCaptureFilePathTemplate pRENDERDOC_GetLogFilePathTemplate;

typedef uint32_t(RENDERDOC_CC *pRENDERDOC_GetNumCaptures)(void);
typedef uint32_t(RENDERDOC_CC *pRENDERDOC_GetCapture)(uint32_t idx, char *filename,
                                                      uint32_t *pathlength, uint64_t *timestamp);

typedef void(RENDERDOC_CC *pRENDERDOC_SetCaptureFileComments)(const char *filePath,
                                                              const char *comments);

typedef uint32_t(RENDERDOC_CC *pRENDERDOC_IsTargetControlConnected)(void);
typedef pRENDERDOC_IsTargetControlConnected pRENDERDOC_IsRemoteAccessConnected;

typedef uint32_t(RENDERDOC_CC *pRENDERDOC_LaunchReplayUI)(uint32_t connectTargetControl,
                                                          const char *cmdline);
typedef void(RENDERDOC_CC *pRENDERDOC_GetAPIVersion)(int *major, int *minor, int *patch);
typedef uint32_t(RENDERDOC_CC *pRENDERDOC_ShowReplayUI)(void);

typedef void(RENDERDOC_CC *pRENDERDOC_SetActiveWindow)(RENDERDOC_DevicePointer device,
                                                       RENDERDOC_WindowHandle wndHandle);

typedef void(RENDERDOC_CC *pRENDERDOC_TriggerCapture)(void);
typedef void(RENDERDOC_CC *pRENDERDOC_TriggerMultiFrameCapture)(uint32_t numFrames);

typedef void(RENDERDOC_CC *pRENDERDOC_StartFrameCapture)(RENDERDOC_DevicePointer device,
                                                         RENDERDOC_WindowHandle wndHandle);
typedef uint32_t(RENDERDOC_CC *pRENDERDOC_IsFrameCapturing)(void);
typedef uint32_t(RENDERDOC_CC *pRENDERDOC_EndFrameCapture)(RENDERDOC_DevicePointer device,
                                                           RENDERDOC_WindowHandle wndHandle);
typedef uint32_t(RENDERDOC_CC *pRENDERDOC_DiscardFrameCapture)(RENDERDOC_DevicePointer device,
                                                               RENDERDOC_WindowHandle wndHandle);

typedef void(RENDERDOC_CC *pRENDERDOC_SetCaptureTitle)(const char *title);

typedef enum RENDERDOC_Version
{
  eRENDERDOC_API_Version_1_6_0 = 10600,
} RENDERDOC_Version;

typedef int(RENDERDOC_CC *pRENDERDOC_GetAPI)(RENDERDOC_Version version, void **outAPIPointers);

typedef struct RENDERDOC_API_1_6_0
{
  pRENDERDOC_GetAPIVersion GetAPIVersion;

  pRENDERDOC_SetCaptureOptionU32 SetCaptureOptionU32;
  pRENDERDOC_SetCaptureOptionF32 SetCaptureOptionF32;

  pRENDERDOC_GetCaptureOptionU32 GetCaptureOptionU32;
  pRENDERDOC_GetCaptureOptionF32 GetCaptureOptionF32;

  pRENDERDOC_SetFocusToggleKeys SetFocusToggleKeys;
  pRENDERDOC_SetCaptureKeys SetCaptureKeys;

  pRENDERDOC_GetOverlayBits GetOverlayBits;
  pRENDERDOC_MaskOverlayBits MaskOverlayBits;

  union
  {
    pRENDERDOC_Shutdown Shutdown;
    pRENDERDOC_RemoveHooks RemoveHooks;
  };

  pRENDERDOC_UnloadCrashHandler UnloadCrashHandler;

  union
  {
    pRENDERDOC_SetLogFilePathTemplate SetLogFilePathTemplate;
    pRENDERDOC_SetCaptureFilePathTemplate SetCaptureFilePathTemplate;
  };

  union
  {
    pRENDERDOC_GetLogFilePathTemplate GetLogFilePathTemplate;
    pRENDERDOC_GetCaptureFilePathTemplate GetCaptureFilePathTemplate;
  };

  pRENDERDOC_GetNumCaptures GetNumCaptures;
  pRENDERDOC_GetCapture GetCapture;

  pRENDERDOC_TriggerCapture TriggerCapture;

  union
  {
    pRENDERDOC_IsRemoteAccessConnected IsRemoteAccessConnected;
    pRENDERDOC_IsTargetControlConnected IsTargetControlConnected;
  };

  pRENDERDOC_LaunchReplayUI LaunchReplayUI;

  pRENDERDOC_SetActiveWindow SetActiveWindow;

  pRENDERDOC_StartFrameCapture StartFrameCapture;
  pRENDERDOC_IsFrameCapturing IsFrameCapturing;
  pRENDERDOC_EndFrameCapture EndFrameCapture;

  pRENDERDOC_TriggerMultiFrameCapture TriggerMultiFrameCapture;

  pRENDERDOC_SetCaptureFileComments SetCaptureFileComments;

  pRENDERDOC_DiscardFrameCapture DiscardFrameCapture;

  pRENDERDOC_ShowReplayUI ShowReplayUI;

  pRENDERDOC_SetCaptureTitle SetCaptureTitle;
} RENDERDOC_API_1_6_0;
