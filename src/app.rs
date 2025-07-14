use anyhow::Result;
use cgmath::*;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window},
};

use crate::{
    camera::CameraWrapper,
    entity::{Entity, EntityData, EntityWrapper},
    light::PointLight,
    texture,
    vertex::{Material, Mesh, Vertex},
};

struct Pass {
    render_pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<wgpu::BindGroup>,
}

struct WgpuContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

struct AppState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    context: WgpuContext,
    start_time: Instant,
    passes: Vec<Pass>,
    camera_wrapper: CameraWrapper,
    entities: Vec<EntityWrapper>,
    depth_texture: wgpu::TextureView,
    camera_uniform_buffer: wgpu::Buffer,
    point_light_uniform_buffer: wgpu::Buffer,
    entities_data_buffer: wgpu::Buffer,
    material: Material,
}

impl AppState {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let (context, surface) = Self::get_wgpu_context_and_surface(window.clone()).await?;

        let surface_caps = surface.get_capabilities(&context.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Data loading and stuff now

        let mut plane = Mesh::new(vec![], vec![]);
        plane.generate_plane(1024);
        let plane_entity = Entity::new(
            plane,
            cgmath::Vector3::zero(),
            cgmath::Quaternion::from_angle_x(Deg(-90.0)),
            cgmath::vec3(2.0, 2.0, 2.0),
        );
        let plane_entity_wrapper = EntityWrapper::new(plane_entity, &context.device);
        let mut cube = Mesh::new(vec![], vec![]);
        cube.generate_cube();
        let cube_entity = Entity::new(
            cube,
            cgmath::Vector3::zero(),
            cgmath::Quaternion::one(),
            cgmath::vec3(0.01, 0.01, 0.01),
        );
        let cube_entity_wrapper = EntityWrapper::new(cube_entity, &context.device);

        let camera_wrapper = CameraWrapper::default();
        let camera_uniform_data = camera_wrapper.get_camera_uniform_data(
            surface_config.width as f32 / surface_config.height as f32,
            0.1,
            100.0,
        );
        let camera_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Uniform"),
                    contents: bytemuck::bytes_of(&camera_uniform_data),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let camera_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let camera_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera_bind_group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                }],
            });
        let point_light = PointLight::new((0.0, 2.0, 0.0, 1.0).into(), (1.0, 1.0, 1.0).into());
        let point_light_uniform_data = point_light.to_uniform_data();
        let point_light_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::bytes_of(&point_light_uniform_data),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let light_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let light_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("light_bind_group"),
                layout: &light_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: point_light_uniform_buffer.as_entire_binding(),
                }],
            });

        let depth_texture = create_depth_texture(&context.device, &surface_config);
        let material_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let material = {
            let diffuse_bytes = include_bytes!("textures/SandTexture.png");
            let diffuse_texture = texture::Texture::from_bytes(
                &context.device,
                &context.queue,
                diffuse_bytes,
                "SandTexture.png",
                wgpu::TextureFormat::Rgba8UnormSrgb,
            )
            .unwrap();
            let normal_bytes = include_bytes!("textures/SandNormal.png");
            let normal_texture = texture::Texture::from_bytes(
                &context.device,
                &context.queue,
                normal_bytes,
                "SandNormal.png",
                wgpu::TextureFormat::Rgba8Unorm,
            )
            .unwrap();
            Material::new(
                &context.device,
                "material",
                diffuse_texture,
                normal_texture,
                &material_bind_group_layout,
            )
        };

        let cube_pass = {
            let shader = context
                .device
                .create_shader_module(wgpu::include_wgsl!("shaders/cube.wgsl"));
            let render_pipeline_layout =
                context
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&camera_bind_group_layout],
                        push_constant_ranges: &[],
                    });
            let render_pipeline =
                context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("cube_render_pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                            buffers: &[Vertex::LAYOUT, EntityData::LAYOUT],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: surface_config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: true,
                            depth_compare: wgpu::CompareFunction::Less,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                        cache: None,
                    });

            Pass {
                render_pipeline,
                bind_groups: vec![camera_bind_group.clone()],
            }
        };
        let plane_pass = {
            let render_pipeline = {
                let shader = context
                    .device
                    .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
                let render_pipeline_layout =
                    context
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[
                                &camera_bind_group_layout,
                                &light_bind_group_layout,
                                &material_bind_group_layout,
                            ],
                            push_constant_ranges: &[],
                        });
                context
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("render_pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                            buffers: &[Vertex::LAYOUT, EntityData::LAYOUT],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: surface_config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: true,
                            depth_compare: wgpu::CompareFunction::Less,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                        cache: None,
                    })
            };

            let bind_groups = vec![
                camera_bind_group.clone(),
                light_bind_group,
                material.bind_group.clone(),
            ];
            Pass {
                render_pipeline,
                bind_groups,
            }
        };

        let entities = vec![cube_entity_wrapper, plane_entity_wrapper];
        let entities_data = entities
            .iter()
            .map(|e| e.to_entity_data())
            .collect::<Vec<_>>();
        let entities_data_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("entities data buffer"),
                    contents: bytemuck::cast_slice(&entities_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Ok(Self {
            window,
            surface,
            surface_config,
            is_surface_configured: false,
            context,
            passes: vec![cube_pass, plane_pass],
            start_time: Instant::now(),
            entities,
            camera_wrapper,
            depth_texture,
            camera_uniform_buffer,
            point_light_uniform_buffer,
            entities_data_buffer,
            material,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let width = size.width.max(1);
        let height = size.height.max(1);
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface
            .configure(&self.context.device, &self.surface_config);
        self.is_surface_configured = true;
        self.depth_texture = create_depth_texture(&self.context.device, &self.surface_config);
    }
    pub fn render(&mut self) -> Result<()> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor::default());
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            for (i, pass) in self.passes.iter().enumerate() {
                render_pass.set_pipeline(&pass.render_pipeline);
                for (i, bind_group) in pass.bind_groups.iter().enumerate() {
                    render_pass.set_bind_group(i as _, bind_group, &[]);
                }
                let entity = &self.entities[i];

                render_pass.set_vertex_buffer(0, entity.mesh_data.vertex_buf.slice(..));
                render_pass.set_vertex_buffer(
                    1,
                    self.entities_data_buffer.slice(
                        ((size_of::<EntityData>() * i) as wgpu::BufferAddress)
                            ..((size_of::<EntityData>() * (i + 1)) as wgpu::BufferAddress),
                    ),
                );
                render_pass.set_index_buffer(
                    entity.mesh_data.index_buf.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..entity.mesh_data.index_len, 0, 0..1);
            }
        }

        self.context.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
    pub fn update(&mut self, dt: Duration) {
        let time_since_start = Instant::now().duration_since(self.start_time);
        let (s, c) = cgmath::Angle::sin_cos(Deg(100.0 * time_since_start.as_secs_f32()));

        let point_light = PointLight::new(
            (s * 0.5 + 1.5, 0.2, c * 0.5 - 1.5, 1.0).into(),
            (0.3, 0.3, 0.3).into(),
        );

        let point_light_uniform_data = point_light.to_uniform_data();
        self.context.queue.write_buffer(
            &self.point_light_uniform_buffer,
            0,
            &bytemuck::bytes_of(&point_light_uniform_data),
        );

        self.entities[0].update_entity_position(point_light.position.xyz());
        let entities_data = self
            .entities
            .iter()
            .map(|e| e.to_entity_data())
            .collect::<Vec<_>>();
        self.context.queue.write_buffer(
            &self.entities_data_buffer,
            0,
            &bytemuck::cast_slice(&entities_data),
        );

        self.camera_wrapper.update(dt);
        let camera_uniform_data = self.camera_wrapper.get_camera_uniform_data(
            self.surface_config.width as f32 / self.surface_config.height as f32,
            0.1,
            100.0,
        );
        self.context.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::bytes_of(&camera_uniform_data),
        );
    }
    async fn get_wgpu_context_and_surface(
        window: Arc<Window>,
    ) -> Result<(WgpuContext, wgpu::Surface<'static>)> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::wgt::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_bind_groups: 8,
                    ..Default::default()
                },
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;
        Ok((
            WgpuContext {
                instance,
                adapter,
                device,
                queue,
            },
            surface,
        ))
    }

    fn handle_keyboard(&mut self, code: KeyCode, key_state: ElementState) {
        self.camera_wrapper.handle_keyboard(code, key_state);
    }

    fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {
        self.camera_wrapper.handle_mouse_motion(dx, dy);
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    depth_texture.create_view(&wgpu::wgt::TextureViewDescriptor::default())
}

struct App {
    state: Option<AppState>,
    last_time: Instant,
}

impl ApplicationHandler<AppState> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
            .unwrap();
        window.set_cursor_visible(false);

        let state = pollster::block_on(AppState::new(window)).unwrap();
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size),
            WindowEvent::RedrawRequested => {
                let dt = self.last_time.elapsed();
                self.last_time = Instant::now();
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    Err(e) => match e.downcast_ref() {
                        Some(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let size = state.window.inner_size();
                            state.resize(size);
                        }
                        Some(e) => {
                            log::error!("Unable to render {}", e);
                        }
                        None => {}
                    },
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                state.handle_keyboard(code, key_state);
                match (code, key_state.is_pressed()) {
                    (KeyCode::Escape, true) => {
                        event_loop.exit();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: AppState) {
        self.state = Some(event);
    }
    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                state.handle_mouse_motion(dx, dy);
            }
            _ => {}
        }
    }
}

impl App {
    fn init() -> Self {
        Self {
            state: None,
            last_time: Instant::now(),
        }
    }
}

pub fn run() -> Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::init();
    event_loop.run_app(&mut app)?;
    Ok(())
}
