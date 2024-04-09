//! This is the main module of the "random-shader-window" project.
//!
//! It contains the main entry point of the application and the `App` struct, which represents the application itself.
//! The `App` struct handles window setup, rendering, and event handling.
//! I'm just using this as a playground to learn how to use wgsl, so it's not really meant to be a useful application.
//!
//! Alot of the render pipeline stuff is using default where possible to keep things simple so
//! I can focus on the shader stuff. It'll probably be a good idea to go back and tweak some of that stuff later
//! to get it to work on other platforms, but I'm not really worried about that right now.

use wgpu::{
	include_wgsl, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
	BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferDescriptor,
	BufferUsages, ColorTargetState, CommandEncoderDescriptor, DeviceDescriptor, Features,
	FragmentState, Instance, Limits, MultisampleState, PipelineLayoutDescriptor, PowerPreference,
	PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
	RequestAdapterOptions, ShaderStages, TextureViewDescriptor, VertexState,
};
use winit::{
	event::{Event, WindowEvent},
	event_loop::EventLoop,
	window::{Window, WindowBuilder},
};

#[tokio::main]
async fn main() {
	// Setup windowing
	let event_loop = EventLoop::new().expect("New event loop");
	let window = WindowBuilder::new()
		.with_title("Random Shader Window")
		.build(&event_loop)
		.expect("New window");

	// Setup the app
	let mut app = App::new(window).await;

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

// impl bytemuck::Pod and bytemuck::Zeroable so we can use it as a buffer
// I'm not sure if this is the best way to do this, but it works.
unsafe impl bytemuck::Pod for MousePosition {}
unsafe impl bytemuck::Zeroable for MousePosition {}

struct App<'a> {
	// Render stuff
	surface: wgpu::Surface<'a>,
	adapter: wgpu::Adapter,
	queue: wgpu::Queue,
	device: wgpu::Device,
	render_pipeline: wgpu::RenderPipeline,

	// Buffer stuff
	buffer: wgpu::Buffer,
	bind_group: wgpu::BindGroup,
	mouse_position: Option<MousePosition>,
}
impl App<'_> {
	async fn new(window: Window) -> Self {
		// Grab the window size now, so we can use it to size our surface.
		// We need it here because we won't have a borrow on the window later.
		let size = window.inner_size();

		// Setting up access to the GPU and the surface to render too.
		let instance = Instance::default();
		let surface = instance.create_surface(window).expect("Create surface");
		let adapter = instance
			.request_adapter(&RequestAdapterOptions {
				power_preference: PowerPreference::default(),
				force_fallback_adapter: false,
				compatible_surface: Some(&surface),
			})
			.await
			.expect("Request adapter");
		let (device, queue) = adapter
			.request_device(
				&DeviceDescriptor {
					label: None,
					required_features: Features::default(),
					required_limits: Limits::default(),
				},
				None,
			)
			.await
			.expect("Request device");
		let surface_config = surface
			.get_default_config(&adapter, size.width, size.height)
			.expect("Get default surface config");
		surface.configure(&device, &surface_config);

		// Set up a buffer to store mouse position
		let buffer = device.create_buffer(&BufferDescriptor {
			label: None,
			size: 32 * 2, // 'vec2<f32>'
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});
		let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::FRAGMENT,
				ty: BindingType::Buffer {
					ty: BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: None,
		});
		let bind_group = device.create_bind_group(&BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
			}],
			label: None,
		});
		let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			bind_group_layouts: &[&bind_group_layout],
			..Default::default()
		});

		// Set up the render pipeline
		let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
		let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: VertexState {
				module: &shader,
				entry_point: "vertex",
				buffers: &[],
			},
			primitive: PrimitiveState::default(),
			depth_stencil: None,
			multisample: MultisampleState::default(),
			fragment: Some(FragmentState {
				module: &shader,
				entry_point: "fragment",
				targets: &[Some(ColorTargetState {
					format: surface_config.format,
					blend: None,
					write_mask: Default::default(),
				})],
			}),
			multiview: None,
		});

		Self {
			surface,
			adapter,
			queue,
			device,
			render_pipeline,
			buffer,
			bind_group,
			mouse_position: None,
		}
	}

	fn render(&self) {
		let surface_texture = self
			.surface
			.get_current_texture()
			.expect("Get current texture");
		let texture_view = surface_texture
			.texture
			.create_view(&TextureViewDescriptor::default());

		let mut command_encoder = self
			.device
			.create_command_encoder(&CommandEncoderDescriptor::default());
		{
			let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
				color_attachments: &[Some(RenderPassColorAttachment {
					view: &texture_view,
					resolve_target: None,
					ops: Default::default(),
				})],
				..Default::default()
			});
			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.bind_group, &[]);
			render_pass.draw(0..6, 0..1);
		}

		// Copy the mouse position into the buffer if we have one
		if let Some(mp) = self.mouse_position {
			self.queue
				.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[mp]))
		}

		self.queue.submit(Some(command_encoder.finish()));
		surface_texture.present();
	}

	fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
		self.surface.configure(
			&self.device,
			&self
				.surface
				.get_default_config(&self.adapter, size.width, size.height)
				.expect("Get default surface config"),
		);
	}
}
