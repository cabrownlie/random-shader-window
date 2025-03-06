use std::sync::Arc;
use wasm_bindgen::prelude::*;

use random_shader_window::app::App;
use winit::{event_loop::EventLoop, window::Window};

#[wasm_bindgen(start)]
pub fn main() {
	let event_loop = EventLoop::new().unwrap();

	let window = Arc::new(
		event_loop
			.create_window(Window::default_attributes())
			.unwrap(),
	);

	#[cfg(not(target_arch = "wasm32"))]
	{
		pollster::block_on(run(event_loop, window));
	}
	#[cfg(target_arch = "wasm32")]
	{
		wasm_bindgen_futures::spawn_local(run(event_loop, window));
	}
}

async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
	let mut app = pollster::block_on(App::new(window));
	let _ = event_loop.run_app(&mut app);
}
