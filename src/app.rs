use std::sync::Arc;

use wgpu::{
	include_wgsl, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
	BufferDescriptor, BufferUsages, PipelineLayoutDescriptor, ShaderStages,
};
use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	window::{Window, WindowId},
};

pub struct App {
	window: Arc<Window>,

	// Render
	device: wgpu::Device,
	queue: wgpu::Queue,
	size: winit::dpi::PhysicalSize<u32>,
	surface: wgpu::Surface<'static>,
	surface_format: wgpu::TextureFormat,
	pipeline: wgpu::RenderPipeline,

	// Buffer
	buffer: wgpu::Buffer,
	bind_group: wgpu::BindGroup,
	mouse_position: Option<MousePosition>,
}

impl App {
	pub async fn new(window: Arc<Window>) -> Self {
		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions::default())
			.await
			.unwrap();
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor::default(),
				None, // Trace path
			)
			.await
			.unwrap();

		let size = window.inner_size();

		let surface = instance.create_surface(window.clone()).unwrap();
		let cap = surface.get_capabilities(&adapter);
		let surface_format = cap.formats[0];

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

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer: &buffer,
					offset: 0,
					size: None,
				}),
			}],
		});

		let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

		let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			bind_group_layouts: &[&bind_group_layout],
			..Default::default()
		});

		let swapchain_capabilities = surface.get_capabilities(&adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: Some("vertex"),
				compilation_options: Default::default(),
				buffers: &[],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fragment"),
				compilation_options: Default::default(),
				targets: &[Some(swapchain_format.into())],
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
			cache: None,
		});

		let state = App {
			window,
			device,
			queue,
			size,
			surface,
			surface_format,
			pipeline,
			buffer,
			bind_group,
			mouse_position: None,
		};

		// Configure surface for the first time
		state.configure_surface();

		state
	}

	fn configure_surface(&self) {
		let surface_config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: self.surface_format,
			// Request compatibility with the sRGB-format texture view we‘re going to create later.
			view_formats: vec![self.surface_format.add_srgb_suffix()],
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			width: self.size.width,
			height: self.size.height,
			desired_maximum_frame_latency: 2,
			present_mode: wgpu::PresentMode::AutoVsync,
		};
		self.surface.configure(&self.device, &surface_config);
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.size = new_size;

		// reconfigure the surface
		self.configure_surface();
	}

	fn render(&mut self) {
		// Create texture view
		let surface_texture = self
			.surface
			.get_current_texture()
			.expect("failed to acquire next swapchain texture");
		let texture_view = surface_texture
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		// Renders screen
		let mut encoder = self.device.create_command_encoder(&Default::default());
		// Create the renderpass which will clear the screen.
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &texture_view,
				resolve_target: None,
				ops: Default::default(),
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
		});

		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_bind_group(0, &self.bind_group, &[]);
		render_pass.draw(0..6, 0..1);

		// End the renderpass.
		drop(render_pass);

		// Copy the mouse position into the buffer if we have one
		if let Some(mp) = self.mouse_position {
			self.queue
				.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[mp]))
		}

		// Submit the command in the queue to execute
		self.queue.submit([encoder.finish()]);
		self.window.pre_present_notify();
		surface_texture.present();
	}
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		self.window = Arc::new(
			event_loop
				.create_window(Window::default_attributes())
				.unwrap(),
		);
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::CursorMoved {
				device_id: _,
				position,
			} => {
				self.mouse_position = Some(MousePosition {
					x: position.x as f32,
					y: position.y as f32,
				});
				self.render();
			}
			WindowEvent::RedrawRequested => {
				self.render();
			}
			WindowEvent::Resized(new_size) => {
				self.resize(new_size);
			}
			_ => (),
		}
	}
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
