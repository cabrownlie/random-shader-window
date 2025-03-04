//! This is the main module of the "random-shader-window" project.
//!
//! It contains the main entry point of the application and the `App` struct, which represents the application itself.
//! The `App` struct handles window setup, rendering, and event handling.
//! I'm just using this as a playground to learn how to use wgsl, so it's not really meant to be a useful application.
//!
//! Alot of the render pipeline stuff is using default where possible to keep things simple so
//! I can focus on the shader stuff. It'll probably be a good idea to go back and tweak some of that stuff later
//! to get it to work on other platforms, but I'm not really worried about that right now.

use winit::{
	event::{Event, WindowEvent},
	event_loop::EventLoop,
	window::WindowBuilder,
};

mod app;

#[tokio::main]
async fn main() {
	// Setup windowing
	let event_loop = EventLoop::new().expect("New event loop");
	let window = WindowBuilder::new()
		.with_title("Random Shader Window")
		.build(&event_loop)
		.expect("New window");

	// Setup the app
	let mut app = app::App::new(window).await;

	// Run the event loop
	let _ = event_loop.run(|event, elwt| {
		if let Event::WindowEvent {
			window_id: _,
			event,
		} = event
		{
			match event {
				WindowEvent::Resized(size) => {
					app.resize(size);
				}
				WindowEvent::RedrawRequested => {
					app.render();
				}
				WindowEvent::CursorMoved {
					device_id: _,
					position,
				} => {
					// Pass the mouse position to the app
					app.mouse_position = Some(MousePosition {
						x: position.x as f32,
						y: position.y as f32,
					});
					// HACK: Force a redraw, because I can't figure out how to cleanly call
					// [window.request_redraw()] without the borrow checker complaining.
					app.render();
				}
				WindowEvent::CloseRequested => {
					elwt.exit();
				}
				_ => {}
			}
		}
	});
}

// This is the struct that we'll use to pass the mouse position to the shader.
#[repr(C)]
#[derive(Copy, Clone)]
struct MousePosition {
	x: f32,
	y: f32,
}
