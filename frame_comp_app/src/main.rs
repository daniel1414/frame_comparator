#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

mod app;
mod comparator;
mod vulkan;

use anyhow::Result;
use app::App;
use log::{error, info};
use winit::dpi::LogicalSize;
use winit::event::{MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

struct WindowApp {
    window: Option<Window>,
    app: Option<App>,
    minimized: bool,
    mouse_left_pressed: bool,
    last_mouse_x: f64,
}

impl WindowApp {
    pub fn new() -> Self {
        Self {
            app: None,
            window: None,
            minimized: false,
            mouse_left_pressed: false,
            last_mouse_x: 20.0_f64,
        }
    }
}

impl winit::application::ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(LogicalSize {
                        width: 1820,
                        height: 1090,
                    })
                    .with_title("Vulkan frame comparator app"),
            )
            .unwrap();
        if self.app.is_none() {
            info!("Window resumed, creating application...");
            match App::create(&window) {
                Ok(app) => self.app = Some(app),
                Err(e) => {
                    error!("Failed to create application: {:?}", e);
                    event_loop.exit();
                }
            }
        }
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let (Some(app), Some(window)) = (&mut self.app, &self.window) {
            match event {
                WindowEvent::RedrawRequested => {
                    if !self.minimized && !event_loop.exiting() {
                        app.render(window).unwrap();
                    }
                }
                WindowEvent::CloseRequested => {
                    app.destroy();
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => {
                    if size.width == 0 && size.height == 0 {
                        self.minimized = true;
                    } else {
                        self.minimized = false;
                        app.update(window, self.last_mouse_x, self.mouse_left_pressed);
                    }
                    window.request_redraw();
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                } => {
                    self.last_mouse_x = position.x;
                    app.update(window, self.last_mouse_x, self.mouse_left_pressed);
                    window.request_redraw();
                }
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                } => {
                    self.mouse_left_pressed = button == MouseButton::Left && state.is_pressed();
                    app.update(window, self.last_mouse_x, self.mouse_left_pressed);
                    window.request_redraw();
                }
                WindowEvent::DroppedFile(buf) => {
                    println!("{}", buf.display());
                }
                _ => (),
            }
        }
    }
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut window_app = WindowApp::new();
    if let Err(e) = event_loop.run_app(&mut window_app) {
        error!("Application event loop returned an error: {:?}", e);
        return Err(e.into());
    }

    Ok(())
}
