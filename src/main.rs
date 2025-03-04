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
