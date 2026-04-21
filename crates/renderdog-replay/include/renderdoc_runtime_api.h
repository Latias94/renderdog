#pragma once

// RenderDoc replay API headers from `third-party/renderdoc`.
//
// On Windows, upstream headers mark several replay APIs as `dllimport`, which would normally
// require linking against `renderdoc.lib`. `renderdog-replay` deliberately uses runtime loading
// instead, so we blank out the import/export macros before including the replay API surface.
#include "apidefs.h"
#undef RENDERDOC_IMPORT_API
#define RENDERDOC_IMPORT_API
#undef RENDERDOC_API
#define RENDERDOC_API
#include "renderdoc_replay.h"
