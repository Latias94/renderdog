use renderdog::RenderDog;
use renderdog_winit::input_button_from_key_code;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

struct App {
    rd: RenderDog,
    window: Option<Window>,
    capturing: bool,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let rd = RenderDog::new()?;
        Ok(Self {
            rd,
            window: None,
            capturing: false,
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs: WindowAttributes = Window::default_attributes().with_title("renderdog-winit");
        match event_loop.create_window(attrs) {
            Ok(w) => {
                self.window = Some(w);
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            Err(e) => {
                eprintln!("failed to create window: {e}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                let code = match event.physical_key {
                    PhysicalKey::Code(c) => c,
                    _ => return,
                };

                // Demo: print mapped RenderDoc key and trigger capture on F12.
                let _mapped = input_button_from_key_code(code);

                if code == KeyCode::F12 {
                    self.trigger_capture();
                }
            }
            WindowEvent::RedrawRequested => {
                if self.capturing {
                    self.finish_capture();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn trigger_capture(&mut self) {
        self.capturing = true;
        #[cfg(windows)]
        {
            if let Some(window) = &self.window {
                let _ = renderdog_winit::start_frame_capture_window(self.rd.inner(), window);
                if let Some(h) = renderdog_winit::renderdoc_window_handle(window) {
                    let _ = self.rd.set_active_window(None, Some(h));
                }
                window.request_redraw();
                return;
            }
        }

        // Fallback: trigger capture without a native window handle.
        let _ = self.rd.trigger_capture();
        self.capturing = false;
    }

    fn finish_capture(&mut self) {
        #[cfg(windows)]
        {
            if let Some(window) = &self.window {
                let _ = renderdog_winit::end_frame_capture_window(self.rd.inner(), window);
            }
        }
        self.capturing = false;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
