#include "replay.h"

#include <atomic>
#include <cstring>
#include <cstdlib>
#include <cstdio>
#include <map>
#include <stdexcept>
#include <string>

#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
#else
#include <dlfcn.h>
#endif

// RenderDoc replay API headers (from submodule: third-party/renderdoc)
//
// NOTE: On Windows, RenderDoc headers declare some APIs as `dllimport`, which normally requires
// linking against `renderdoc.lib`. For this experimental crate we want to be purely runtime-loaded,
// so we:
//   1) include apidefs.h once to get platform defines like RENDERDOC_CC
//   2) override RENDERDOC_IMPORT_API/RENDERDOC_API to be blank for subsequent headers
//   3) provide local stubs for `RENDERDOC_AllocArrayMem` / `RENDERDOC_FreeArrayMem` that forward to
//      the real functions resolved from the loaded renderdoc module.
#include "apidefs.h"
#undef RENDERDOC_IMPORT_API
#define RENDERDOC_IMPORT_API
#undef RENDERDOC_API
#define RENDERDOC_API
#include "renderdoc_replay.h"

// Mark this process as a replay program so RenderDoc doesn't try to capture or hook itself.
REPLAY_PROGRAM_MARKER();

namespace renderdog {
namespace replay {

namespace {

using pRENDERDOC_AllocArrayMem = void *(RENDERDOC_CC *)(uint64_t);
using pRENDERDOC_FreeArrayMem = void(RENDERDOC_CC *)(void *);

#if defined(_WIN32)
using RenderdocModule = HMODULE;
static std::atomic<HMODULE> g_renderdoc_module{NULL};
#else
using RenderdocModule = void *;
static std::atomic<void *> g_renderdoc_module{nullptr};
#endif

static std::atomic<pRENDERDOC_AllocArrayMem> g_alloc_array_mem{nullptr};
static std::atomic<pRENDERDOC_FreeArrayMem> g_free_array_mem{nullptr};

static RenderdocModule get_renderdoc_module()
{
#if defined(_WIN32)
  HMODULE m = g_renderdoc_module.load();
  if(m)
    return m;

  if(const char *dll = std::getenv("RENDERDOG_REPLAY_RENDERDOC_DLL"))
  {
    HMODULE lib = LoadLibraryA(dll);
    if(lib)
    {
      g_renderdoc_module.store(lib);
      return lib;
    }
  }

  if(const char *dir = std::getenv("RENDERDOG_RENDERDOC_DIR"))
  {
    std::string path(dir);
    if(!path.empty() && path.back() != '\\' && path.back() != '/')
      path.push_back('\\');
    path += "renderdoc.dll";
    HMODULE lib = LoadLibraryA(path.c_str());
    if(lib)
    {
      g_renderdoc_module.store(lib);
      return lib;
    }
  }

  HMODULE lib = LoadLibraryA("renderdoc.dll");
  if(lib)
  {
    g_renderdoc_module.store(lib);
    return lib;
  }

  return NULL;
#else
  void *m = g_renderdoc_module.load();
  if(m)
    return m;

  if(const char *so = std::getenv("RENDERDOG_REPLAY_RENDERDOC_SO"))
  {
    void *lib = dlopen(so, RTLD_NOW | RTLD_LOCAL);
    if(lib)
    {
      g_renderdoc_module.store(lib);
      return lib;
    }
  }

  if(const char *dir = std::getenv("RENDERDOG_RENDERDOC_DIR"))
  {
    std::string base(dir);
    if(!base.empty() && base.back() != '/')
      base.push_back('/');
    for(const char *name : {"librenderdoc.so.1", "librenderdoc.so"})
    {
      std::string path = base + name;
      void *lib = dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL);
      if(lib)
      {
        g_renderdoc_module.store(lib);
        return lib;
      }
    }
  }

  for(const char *name : {"librenderdoc.so.1", "librenderdoc.so"})
  {
    void *lib = dlopen(name, RTLD_NOW | RTLD_LOCAL);
    if(lib)
    {
      g_renderdoc_module.store(lib);
      return lib;
    }
  }

  return nullptr;
#endif
}

static void ensure_array_allocators()
{
  if(g_alloc_array_mem.load() && g_free_array_mem.load())
    return;

  RenderdocModule m = get_renderdoc_module();
  if(!m)
    throw std::runtime_error("RenderDoc module not loaded (cannot resolve array allocators)");

#if defined(_WIN32)
  auto alloc =
      reinterpret_cast<pRENDERDOC_AllocArrayMem>(GetProcAddress(m, "RENDERDOC_AllocArrayMem"));
  auto free =
      reinterpret_cast<pRENDERDOC_FreeArrayMem>(GetProcAddress(m, "RENDERDOC_FreeArrayMem"));
#else
  auto alloc = reinterpret_cast<pRENDERDOC_AllocArrayMem>(dlsym(m, "RENDERDOC_AllocArrayMem"));
  auto free = reinterpret_cast<pRENDERDOC_FreeArrayMem>(dlsym(m, "RENDERDOC_FreeArrayMem"));
#endif

  if(!alloc || !free)
    throw std::runtime_error("Failed to resolve RENDERDOC_AllocArrayMem/RENDERDOC_FreeArrayMem");

  g_alloc_array_mem.store(alloc);
  g_free_array_mem.store(free);
}

static bool trace_enabled()
{
  const char *v = std::getenv("RENDERDOG_REPLAY_TRACE");
  return v && v[0] && v[0] != '0';
}

static bool trace_alloc_enabled()
{
  const char *v = std::getenv("RENDERDOG_REPLAY_TRACE_ALLOC");
  return v && v[0] && v[0] != '0';
}

static void trace(const char *msg)
{
  if(!trace_enabled())
    return;
  std::fprintf(stderr, "[renderdog-replay] %s\n", msg);
  std::fflush(stderr);
}

template <typename T>
T load_symbol(void *lib, const char *name)
{
#if defined(_WIN32)
  FARPROC sym = GetProcAddress((HMODULE)lib, name);
  if(!sym)
    throw std::runtime_error(std::string("missing symbol: ") + name);
  return reinterpret_cast<T>(sym);
#else
  void *sym = dlsym(lib, name);
  if(!sym)
    throw std::runtime_error(std::string("missing symbol: ") + name);
  return reinterpret_cast<T>(sym);
#endif
}

std::string json_escape(const rdcstr &s)
{
  const char *p = s.c_str();
  std::string out;
  out.reserve(s.size() + 8);
  for(size_t i = 0; i < s.size(); i++)
  {
    const char c = p[i];
    if(c == '\\')
      out += "\\\\";
    else if(c == '"')
      out += "\\\"";
    else if(c == '\n')
      out += "\\n";
    else if(c == '\r')
      out += "\\r";
    else if(c == '\t')
      out += "\\t";
    else
      out.push_back(c);
  }
  return out;
}

} // namespace

extern "C" void *RENDERDOC_CC RENDERDOC_AllocArrayMem(uint64_t sz)
{
  try
  {
    if(trace_enabled() && trace_alloc_enabled())
      trace("RENDERDOC_AllocArrayMem");
    ensure_array_allocators();
    auto f = g_alloc_array_mem.load();
    return f ? f(sz) : nullptr;
  }
  catch(...)
  {
    return nullptr;
  }
}

extern "C" void RENDERDOC_CC RENDERDOC_FreeArrayMem(void *mem)
{
  try
  {
    if(trace_enabled() && trace_alloc_enabled())
      trace("RENDERDOC_FreeArrayMem");
    ensure_array_allocators();
    auto f = g_free_array_mem.load();
    if(f)
      f(mem);
  }
  catch(...)
  {
  }
}

std::unique_ptr<ReplaySession> replay_session_new(rust::Str renderdoc_path)
{
  auto sess = std::make_unique<ReplaySession>();
  if(renderdoc_path.size() > 0)
  {
    // We store the path via environment? For now just load eagerly from this path.
    // This keeps behaviour deterministic for experiments.
    std::string path(renderdoc_path.data(), renderdoc_path.size());

#if defined(_WIN32)
    HMODULE lib = LoadLibraryA(path.c_str());
    if(!lib)
      throw std::runtime_error("LoadLibraryA failed");
    sess->lib_ = (void *)lib;
    g_renderdoc_module.store(lib);
#else
    void *lib = dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL);
    if(!lib)
      throw std::runtime_error(std::string("dlopen failed: ") + dlerror());
    sess->lib_ = lib;
    g_renderdoc_module.store(lib);
#endif
  }

  return sess;
}

ReplaySession::~ReplaySession()
{
  if(controller_)
  {
    controller_->Shutdown();
    controller_ = nullptr;
  }

  if(capture_file_)
  {
    capture_file_->Shutdown();
    capture_file_ = nullptr;
  }

  if(replay_initialised_)
  {
    // If we can, call ShutdownReplay. This is best-effort.
    using pRENDERDOC_ShutdownReplay = void(RENDERDOC_CC *)();
    try
    {
      auto sym = load_symbol<pRENDERDOC_ShutdownReplay>(lib_, "RENDERDOC_ShutdownReplay");
      sym();
    }
    catch(...)
    {
    }
    replay_initialised_ = false;
  }

  if(lib_)
  {
#if defined(_WIN32)
    FreeLibrary((HMODULE)lib_);
#else
    dlclose(lib_);
#endif
    lib_ = nullptr;
  }
}

void ReplaySession::ensure_loaded()
{
  if(lib_)
    return;

#if defined(_WIN32)
  if(const char *dll = std::getenv("RENDERDOG_REPLAY_RENDERDOC_DLL"))
  {
    HMODULE lib = LoadLibraryA(dll);
    if(lib)
    {
      lib_ = (void *)lib;
      g_renderdoc_module.store(lib);
      return;
    }
  }

  if(const char *dir = std::getenv("RENDERDOG_RENDERDOC_DIR"))
  {
    std::string path(dir);
    if(!path.empty() && path.back() != '\\' && path.back() != '/')
      path.push_back('\\');
    path += "renderdoc.dll";
    HMODULE lib = LoadLibraryA(path.c_str());
    if(lib)
    {
      lib_ = (void *)lib;
      g_renderdoc_module.store(lib);
      return;
    }
  }

  const char *candidates[] = {"renderdoc.dll"};
  for(const char *name : candidates)
  {
    HMODULE lib = LoadLibraryA(name);
    if(lib)
    {
      lib_ = (void *)lib;
      g_renderdoc_module.store(lib);
      return;
    }
  }
  throw std::runtime_error("failed to load renderdoc.dll (set explicit path)");
#else
  if(const char *so = std::getenv("RENDERDOG_REPLAY_RENDERDOC_SO"))
  {
    void *lib = dlopen(so, RTLD_NOW | RTLD_LOCAL);
    if(lib)
    {
      lib_ = lib;
      g_renderdoc_module.store(lib);
      return;
    }
  }

  if(const char *dir = std::getenv("RENDERDOG_RENDERDOC_DIR"))
  {
    std::string base(dir);
    if(!base.empty() && base.back() != '/')
      base.push_back('/');
    for(const char *name : {"librenderdoc.so.1", "librenderdoc.so"})
    {
      std::string path = base + name;
      void *lib = dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL);
      if(lib)
      {
        lib_ = lib;
        g_renderdoc_module.store(lib);
        return;
      }
    }
  }

  const char *candidates[] = {"librenderdoc.so", "librenderdoc.so.1"};
  for(const char *name : candidates)
  {
    void *lib = dlopen(name, RTLD_NOW | RTLD_LOCAL);
    if(lib)
    {
      lib_ = lib;
      g_renderdoc_module.store(lib);
      return;
    }
  }
  throw std::runtime_error(
      "failed to load librenderdoc.so (install RenderDoc or set explicit path)");
#endif
}

void ReplaySession::open_capture(rust::Str capture_path)
{
  trace("open_capture: begin");
  ensure_loaded();
  trace("open_capture: ensure_loaded ok");

  if(!replay_initialised_)
  {
    trace("open_capture: init replay");
    using pRENDERDOC_InitialiseReplay = void(RENDERDOC_CC *)(GlobalEnvironment, const rdcarray<rdcstr> &);
    auto init = load_symbol<pRENDERDOC_InitialiseReplay>(lib_, "RENDERDOC_InitialiseReplay");

    GlobalEnvironment env;
    rdcarray<rdcstr> args;
    init(env, args);
    replay_initialised_ = true;
    trace("open_capture: init replay ok");
  }

  trace("open_capture: open capture file");
  using pRENDERDOC_OpenCaptureFile = ICaptureFile *(RENDERDOC_CC *)();
  auto open_file = load_symbol<pRENDERDOC_OpenCaptureFile>(lib_, "RENDERDOC_OpenCaptureFile");

  capture_file_ = open_file();
  if(!capture_file_)
    throw std::runtime_error("RENDERDOC_OpenCaptureFile returned null");
  trace("open_capture: open capture file ok");

  rdcstr filename(std::string(capture_path.data(), capture_path.size()).c_str());
  trace("open_capture: OpenFile");
  ResultDetails open_res = capture_file_->OpenFile(filename, rdcstr("rdc"), nullptr);
  if(!open_res.OK())
  {
    throw std::runtime_error("OpenFile failed");
  }
  trace("open_capture: OpenFile ok");

  trace("open_capture: OpenCapture");
  ReplayOptions opts;
  auto pair = capture_file_->OpenCapture(opts, nullptr);
  if(!pair.first.OK() || pair.second == nullptr)
  {
    throw std::runtime_error("OpenCapture failed");
  }

  controller_ = pair.second;
  trace("open_capture: OpenCapture ok");
}

void ReplaySession::ensure_opened() const
{
  if(!replay_initialised_)
    throw std::runtime_error("replay not initialised");
  if(!capture_file_)
    throw std::runtime_error("capture not opened (call open_capture first)");
  if(!controller_)
    throw std::runtime_error("replay controller not available");
}

void ReplaySession::set_frame_event(uint32_t event_id)
{
  ensure_opened();
  controller_->SetFrameEvent(event_id, true);
}

rust::String ReplaySession::list_textures_json() const
{
  ensure_opened();

  const auto &resources = controller_->GetResources();
  std::map<ResourceId, rdcstr> name_by_id;
  name_by_id.clear();
  for(size_t i = 0; i < resources.size(); i++)
    name_by_id[resources[i].resourceId] = resources[i].name;

  const auto &textures = controller_->GetTextures();
  std::string out = "[";
  for(size_t i = 0; i < textures.size(); i++)
  {
    const auto &t = textures[i];
    auto it = name_by_id.find(t.resourceId);
    rdcstr name = it != name_by_id.end() ? it->second : rdcstr("<unknown>");

    if(i > 0)
      out += ",";

    out += "{";
    out += "\"index\":";
    out += std::to_string((uint32_t)i);
    out += ",\"name\":\"";
    out += json_escape(name);
    out += "\"";
    out += ",\"width\":";
    out += std::to_string(t.width);
    out += ",\"height\":";
    out += std::to_string(t.height);
    out += ",\"depth\":";
    out += std::to_string(t.depth);
    out += ",\"mips\":";
    out += std::to_string(t.mips);
    out += ",\"arraysize\":";
    out += std::to_string(t.arraysize);
    out += ",\"msSamp\":";
    out += std::to_string(t.msSamp);
    out += ",\"byteSize\":";
    out += std::to_string((uint64_t)t.byteSize);
    out += "}";
  }
  out += "]";
  return rust::String(out);
}

rust::Vec<float> ReplaySession::pick_pixel(uint32_t texture_index, uint32_t x, uint32_t y) const
{
  ensure_opened();

  const auto &textures = controller_->GetTextures();
  if(texture_index >= textures.size())
    throw std::runtime_error("texture_index out of range");

  const auto &t = textures[texture_index];
  Subresource sub(0, 0, 0);
  PixelValue pv = controller_->PickPixel(t.resourceId, x, y, sub, CompType::Typeless);

  rust::Vec<float> out;
  out.reserve(4);
  out.push_back(pv.floatValue[0]);
  out.push_back(pv.floatValue[1]);
  out.push_back(pv.floatValue[2]);
  out.push_back(pv.floatValue[3]);
  return out;
}

void ReplaySession::save_texture_png(uint32_t texture_index, rust::Str output_path) const
{
  ensure_opened();

  const auto &textures = controller_->GetTextures();
  if(texture_index >= textures.size())
    throw std::runtime_error("texture_index out of range");

  const auto &t = textures[texture_index];

  TextureSave save;
  save.resourceId = t.resourceId;
  save.destType = FileType::PNG;
  save.mip = 0;

  rdcstr out_path(std::string(output_path.data(), output_path.size()).c_str());
  ResultDetails res = controller_->SaveTexture(save, out_path);
  if(!res.OK())
  {
    throw std::runtime_error("SaveTexture failed");
  }
}

} // namespace replay
} // namespace renderdog
