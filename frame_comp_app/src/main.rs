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
use vulkanalia::prelude::v1_3::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, MouseButton, WindowEvent};
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
    let mut left_mouse_pressed = false;
    let mut mouse_x: f64 = (window.inner_size().width / 2) as f64;

    event_loop.run(move |event, elwt| {
        let check_and_update_app = |mouse: f64, mouse_pressed: bool, app: &mut App| {
            let window_size = app.data.window_size;
            if mouse_pressed && mouse < window_size.width as f64 - 10.0 && mouse > 10.0 {
                app.data.vbar_percentage = mouse / window_size.width as f64;
                app.resized = true;
            }
        };
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
                            app.data.window_size = size;
                        }
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                    } => {
                        mouse_x = position.x;
                        check_and_update_app(mouse_x, left_mouse_pressed, &mut app);
                    }
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                    } => {
                        left_mouse_pressed = button == MouseButton::Left && state.is_pressed();
                        check_and_update_app(mouse_x, left_mouse_pressed, &mut app);
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
