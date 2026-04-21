#include "replay.h"

#include <atomic>
#include <cstring>
#include <cstdlib>
#include <cstdio>
#include <map>
#include <mutex>
#include <stdexcept>
#include <string>

#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
#else
#include <dlfcn.h>
#endif

// Keep RenderDoc header import overrides in one shim so submodule upgrades only touch one file.
#include "renderdoc_runtime_api.h"

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
static std::mutex g_replay_runtime_mutex;
static size_t g_replay_runtime_users = 0;

static RenderdocModule store_renderdoc_module(RenderdocModule module)
{
  if(module)
  {
    RenderdocModule existing = g_renderdoc_module.load();
    if(existing && existing != module)
    {
      throw std::runtime_error(
          "renderdog-replay only supports one RenderDoc module per process; restart the process before switching installations");
    }

    g_renderdoc_module.store(module);
  }
  return module;
}

static void *renderdoc_module_ptr(RenderdocModule module)
{
#if defined(_WIN32)
  return reinterpret_cast<void *>(module);
#else
  return module;
#endif
}

static RenderdocModule load_renderdoc_module(const char *path)
{
#if defined(_WIN32)
  return path ? LoadLibraryA(path) : NULL;
#else
  return path ? dlopen(path, RTLD_NOW | RTLD_LOCAL) : nullptr;
#endif
}

static RenderdocModule load_renderdoc_module_or_throw(const char *path)
{
  RenderdocModule module = load_renderdoc_module(path);
  if(module)
    return store_renderdoc_module(module);

#if defined(_WIN32)
  throw std::runtime_error(std::string("LoadLibraryA failed: ") + path);
#else
  const char *err = dlerror();
  std::string message = std::string("dlopen failed: ") + path;
  if(err && err[0])
    message += std::string(" (") + err + ")";
  throw std::runtime_error(message);
#endif
}

static RenderdocModule try_load_default_renderdoc_module()
{
#if defined(_WIN32)
  if(const char *dll = std::getenv("RENDERDOG_REPLAY_RENDERDOC_DLL"))
  {
    if(RenderdocModule module = load_renderdoc_module(dll))
      return store_renderdoc_module(module);
  }

  if(const char *dir = std::getenv("RENDERDOG_RENDERDOC_DIR"))
  {
    std::string path(dir);
    if(!path.empty() && path.back() != '\\' && path.back() != '/')
      path.push_back('\\');
    path += "renderdoc.dll";
    if(RenderdocModule module = load_renderdoc_module(path.c_str()))
      return store_renderdoc_module(module);
  }

  return store_renderdoc_module(load_renderdoc_module("renderdoc.dll"));
#else
  if(const char *so = std::getenv("RENDERDOG_REPLAY_RENDERDOC_SO"))
  {
    if(RenderdocModule module = load_renderdoc_module(so))
      return store_renderdoc_module(module);
  }

  if(const char *dir = std::getenv("RENDERDOG_RENDERDOC_DIR"))
  {
    std::string base(dir);
    if(!base.empty() && base.back() != '/')
      base.push_back('/');
    for(const char *name : {"librenderdoc.so.1", "librenderdoc.so"})
    {
      std::string path = base + name;
      if(RenderdocModule module = load_renderdoc_module(path.c_str()))
        return store_renderdoc_module(module);
    }
  }

  for(const char *name : {"librenderdoc.so.1", "librenderdoc.so"})
  {
    if(RenderdocModule module = load_renderdoc_module(name))
      return store_renderdoc_module(module);
  }

  return nullptr;
#endif
}

static RenderdocModule get_renderdoc_module()
{
  RenderdocModule module = g_renderdoc_module.load();
  if(module)
    return module;
  return try_load_default_renderdoc_module();
}

static RenderdocModule ensure_renderdoc_module_loaded(const char *path)
{
  if(path && path[0])
    return store_renderdoc_module(load_renderdoc_module_or_throw(path));

  RenderdocModule module = get_renderdoc_module();
  if(module)
    return module;

#if defined(_WIN32)
  throw std::runtime_error("failed to load renderdoc.dll (set explicit path)");
#else
  throw std::runtime_error(
      "failed to load librenderdoc.so (install RenderDoc or set explicit path)");
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

std::string runtime_version_string_from_module(void *lib)
{
  using pRENDERDOC_GetVersionString = const char *(RENDERDOC_CC *)();
  auto get_version =
      load_symbol<pRENDERDOC_GetVersionString>(lib, "RENDERDOC_GetVersionString");
  const char *version = get_version();
  if(!version || !version[0])
    throw std::runtime_error("RENDERDOC_GetVersionString returned an empty version");
  return std::string(version);
}

std::runtime_error result_error(const char *operation, const ResultDetails &result)
{
  std::string detail;
  if(result.internal_msg)
    detail = result.internal_msg->c_str();
  else
    detail = "ResultCode(" + std::to_string(static_cast<uint32_t>(result.code)) + ")";

  return std::runtime_error(std::string(operation) + ": " + detail);
}

std::runtime_error texture_index_error(uint32_t texture_index, size_t texture_count)
{
  return std::runtime_error("texture_index out of range: " + std::to_string(texture_index) +
                            " >= " + std::to_string(texture_count));
}

uint64_t resource_id_to_u64(const ResourceId &resource_id)
{
  static_assert(sizeof(ResourceId) == sizeof(uint64_t), "ResourceId is expected to be 64-bit");

  uint64_t value = 0;
  std::memcpy(&value, &resource_id, sizeof(value));
  return value;
}

void acquire_replay_runtime(void *lib)
{
  std::lock_guard<std::mutex> lock(g_replay_runtime_mutex);
  if(g_replay_runtime_users == 0)
  {
    using pRENDERDOC_InitialiseReplay =
        void(RENDERDOC_CC *)(GlobalEnvironment, const rdcarray<rdcstr> &);
    auto init = load_symbol<pRENDERDOC_InitialiseReplay>(lib, "RENDERDOC_InitialiseReplay");

    GlobalEnvironment env;
    rdcarray<rdcstr> args;
    init(env, args);
  }

  g_replay_runtime_users++;
}

void release_replay_runtime(void *lib)
{
  std::lock_guard<std::mutex> lock(g_replay_runtime_mutex);
  if(g_replay_runtime_users == 0)
    return;

  g_replay_runtime_users--;
  if(g_replay_runtime_users != 0)
    return;

  using pRENDERDOC_ShutdownReplay = void(RENDERDOC_CC *)();
  auto shutdown = load_symbol<pRENDERDOC_ShutdownReplay>(lib, "RENDERDOC_ShutdownReplay");
  shutdown();
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

std::unique_ptr<ReplaySession> replay_session_new_current()
{
  auto sess = std::make_unique<ReplaySession>();
  sess->lib_ = renderdoc_module_ptr(ensure_renderdoc_module_loaded(nullptr));

  return sess;
}

rust::String replay_runtime_probe(rust::Str renderdoc_path)
{
  std::string path(renderdoc_path.data(), renderdoc_path.size());
  RenderdocModule module = ensure_renderdoc_module_loaded(path.c_str());
  return runtime_version_string_from_module(renderdoc_module_ptr(module));
}

ReplaySession::~ReplaySession()
{
  close_capture_state();

  if(replay_runtime_acquired_)
  {
    try
    {
      release_replay_runtime(lib_);
    }
    catch(...)
    {
    }
    replay_runtime_acquired_ = false;
  }

  // The allocator trampolines cache process-global function pointers into the loaded
  // RenderDoc module. Keep the module alive for process lifetime once acquired so
  // those cached pointers never outlive the backing library.
  lib_ = nullptr;
}

void ReplaySession::ensure_loaded()
{
  if(lib_)
    return;
  lib_ = renderdoc_module_ptr(ensure_renderdoc_module_loaded(nullptr));
}

void ReplaySession::close_capture_state()
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
}

void ReplaySession::open_capture(rust::Str capture_path)
{
  trace("open_capture: begin");
  ensure_loaded();
  trace("open_capture: ensure_loaded ok");

  if(!replay_runtime_acquired_)
  {
    trace("open_capture: init replay");
    acquire_replay_runtime(lib_);
    replay_runtime_acquired_ = true;
    trace("open_capture: init replay ok");
  }

  close_capture_state();

  trace("open_capture: open capture file");
  using pRENDERDOC_OpenCaptureFile = ICaptureFile *(RENDERDOC_CC *)();
  auto open_file = load_symbol<pRENDERDOC_OpenCaptureFile>(lib_, "RENDERDOC_OpenCaptureFile");

  ICaptureFile *capture_file = open_file();
  if(!capture_file)
    throw std::runtime_error("RENDERDOC_OpenCaptureFile returned null");
  trace("open_capture: open capture file ok");

  rdcstr filename(std::string(capture_path.data(), capture_path.size()).c_str());
  trace("open_capture: OpenFile");
  ResultDetails open_res = capture_file->OpenFile(filename, rdcstr("rdc"), nullptr);
  if(!open_res.OK())
  {
    capture_file->Shutdown();
    throw result_error("OpenFile failed", open_res);
  }
  trace("open_capture: OpenFile ok");

  trace("open_capture: OpenCapture");
  ReplayOptions opts;
  auto pair = capture_file->OpenCapture(opts, nullptr);
  if(!pair.first.OK() || pair.second == nullptr)
  {
    if(pair.second)
      pair.second->Shutdown();
    capture_file->Shutdown();
    throw result_error("OpenCapture failed", pair.first);
  }

  capture_file_ = capture_file;
  controller_ = pair.second;
  trace("open_capture: OpenCapture ok");
}

void ReplaySession::ensure_opened() const
{
  if(!replay_runtime_acquired_)
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

rust::String ReplaySession::list_textures_serialized() const
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
    out += ",\"resource_id\":";
    out += std::to_string(resource_id_to_u64(t.resourceId));
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
    out += ",\"array_size\":";
    out += std::to_string(t.arraysize);
    out += ",\"ms_samp\":";
    out += std::to_string(t.msSamp);
    out += ",\"byte_size\":";
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
    throw texture_index_error(texture_index, textures.size());

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
    throw texture_index_error(texture_index, textures.size());

  const auto &t = textures[texture_index];

  TextureSave save;
  save.resourceId = t.resourceId;
  save.destType = FileType::PNG;
  save.mip = 0;

  rdcstr out_path(std::string(output_path.data(), output_path.size()).c_str());
  ResultDetails res = controller_->SaveTexture(save, out_path);
  if(!res.OK())
  {
    throw result_error("SaveTexture failed", res);
  }
}

} // namespace replay
} // namespace renderdog
