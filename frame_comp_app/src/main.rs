#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

mod app;
mod vulkan;

use anyhow::Result;
use app::App;
use frame_comp;
use vulkanalia::prelude::v1_3::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Vulkan here we goo!!!")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // Vulkan App
    let mut app = App::create(&window)?;
    let mut minimized = false;
    event_loop.run(move |event, elwt| {
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::RedrawRequested if !elwt.exiting() && !minimized => {
                        app.render(&window).unwrap()
                    }
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                        // Wait for the GPU to finish it's work before we destroy the app
                        // not to destroy components that are currently in use by the GPU.
                        unsafe {
                            app.device.device_wait_idle().unwrap();
                        }

                        // Deallocate everything from the GPU.

                        app.destroy();
                    }
                    WindowEvent::Resized(size) => {
                        if size.width == 0 && size.height == 0 {
                            minimized = true;
                        } else {
                            minimized = false;
                            app.resized = true;
                        }
                    }
                    WindowEvent::DroppedFile(buf) => {
                        println!("{}", buf.display());
                    }
                    _ => (),
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
