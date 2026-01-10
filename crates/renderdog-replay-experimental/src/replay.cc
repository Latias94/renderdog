#include "replay.h"

#include <cstring>
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
#include "renderdoc_replay.h"

namespace renderdog {
namespace replay {

namespace {

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
#else
    void *lib = dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL);
    if(!lib)
      throw std::runtime_error(std::string("dlopen failed: ") + dlerror());
    sess->lib_ = lib;
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
  const char *candidates[] = {"renderdoc.dll"};
  for(const char *name : candidates)
  {
    HMODULE lib = LoadLibraryA(name);
    if(lib)
    {
      lib_ = (void *)lib;
      return;
    }
  }
  throw std::runtime_error("failed to load renderdoc.dll (set explicit path)");
#else
  const char *candidates[] = {"librenderdoc.so", "librenderdoc.so.1"};
  for(const char *name : candidates)
  {
    void *lib = dlopen(name, RTLD_NOW | RTLD_LOCAL);
    if(lib)
    {
      lib_ = lib;
      return;
    }
  }
  throw std::runtime_error(
      "failed to load librenderdoc.so (install RenderDoc or set explicit path)");
#endif
}

void ReplaySession::open_capture(rust::Str capture_path)
{
  ensure_loaded();

  if(!replay_initialised_)
  {
    using pRENDERDOC_InitialiseReplay = void(RENDERDOC_CC *)(GlobalEnvironment, const rdcarray<rdcstr> &);
    auto init = load_symbol<pRENDERDOC_InitialiseReplay>(lib_, "RENDERDOC_InitialiseReplay");

    GlobalEnvironment env;
    rdcarray<rdcstr> args;
    init(env, args);
    replay_initialised_ = true;
  }

  using pRENDERDOC_OpenCaptureFile = ICaptureFile *(RENDERDOC_CC *)();
  auto open_file = load_symbol<pRENDERDOC_OpenCaptureFile>(lib_, "RENDERDOC_OpenCaptureFile");

  capture_file_ = open_file();
  if(!capture_file_)
    throw std::runtime_error("RENDERDOC_OpenCaptureFile returned null");

  rdcstr filename(std::string(capture_path.data(), capture_path.size()).c_str());
  ResultDetails open_res = capture_file_->OpenFile(filename, rdcstr("rdc"), nullptr);
  if(!open_res.OK())
  {
    std::string msg("OpenFile failed: ");
    msg += std::string(open_res.Message().c_str());
    throw std::runtime_error(msg);
  }

  ReplayOptions opts;
  auto pair = capture_file_->OpenCapture(opts, nullptr);
  if(!pair.first.OK() || pair.second == nullptr)
  {
    std::string msg("OpenCapture failed: ");
    msg += std::string(pair.first.Message().c_str());
    throw std::runtime_error(msg);
  }

  controller_ = pair.second;
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

PixelRgba ReplaySession::pick_pixel(uint32_t texture_index, uint32_t x, uint32_t y) const
{
  ensure_opened();

  const auto &textures = controller_->GetTextures();
  if(texture_index >= textures.size())
    throw std::runtime_error("texture_index out of range");

  const auto &t = textures[texture_index];
  Subresource sub(0, 0, 0);
  PixelValue pv = controller_->PickPixel(t.resourceId, x, y, sub, CompType::Typeless);

  PixelRgba out;
  out.r = pv.floatValue[0];
  out.g = pv.floatValue[1];
  out.b = pv.floatValue[2];
  out.a = pv.floatValue[3];
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
    std::string msg("SaveTexture failed: ");
    msg += std::string(res.Message().c_str());
    throw std::runtime_error(msg);
  }
}

} // namespace replay
} // namespace renderdog
