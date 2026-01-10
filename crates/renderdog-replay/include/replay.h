#pragma once

#include <memory>

#include "rust/cxx.h"

struct ICaptureFile;
struct IReplayController;

namespace renderdog {
namespace replay {

class ReplaySession;

std::unique_ptr<ReplaySession> replay_session_new(rust::Str renderdoc_path);

class ReplaySession
{
public:
  ReplaySession() = default;
  ReplaySession(const ReplaySession &) = delete;
  ReplaySession &operator=(const ReplaySession &) = delete;

  ~ReplaySession();

  void open_capture(rust::Str capture_path);
  void set_frame_event(uint32_t event_id);

  rust::String list_textures_json() const;
  rust::Vec<float> pick_pixel(uint32_t texture_index, uint32_t x, uint32_t y) const;
  void save_texture_png(uint32_t texture_index, rust::Str output_path) const;

private:
  friend std::unique_ptr<ReplaySession> replay_session_new(rust::Str renderdoc_path);

  void ensure_loaded();
  void ensure_opened() const;

  void *lib_ = nullptr;
  bool replay_initialised_ = false;

  ::ICaptureFile *capture_file_ = nullptr;
  ::IReplayController *controller_ = nullptr;
};

} // namespace replay
} // namespace renderdog
