use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    CommandEncoderDescriptor, PipelineLayoutDescriptor, RenderPassColorAttachmentDescriptor,
    RenderPassDescriptor, RenderPipelineDescriptor,
};

use crate::solver::{self, Solver};

pub struct Engine {
    window_size: winit::dpi::PhysicalSize<u32>,
    swap_chain: wgpu::SwapChain,
    rt: tokio::runtime::Runtime,
    sph_solver: Solver,
    pipeline: wgpu::RenderPipeline,
    device: wgpu::Device,
    queue: wgpu::Queue,
    particle_buffer: wgpu::Buffer,
}

impl Engine {
    pub async fn new(window: &winit::window::Window) -> Self {
        let window_size = window.inner_size();
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let vertex_stage =
            device.create_shader_module(&wgpu::include_spirv!("shader/shader.vert.spv"));
        let frag_stage =
            device.create_shader_module(&wgpu::include_spirv!("shader/shader.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_stage,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<solver::Particle>() as u64,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![ 0 => Float2 ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_stage,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
        });

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .worker_threads(4)
            .build()
            .unwrap();

        let sph_solver = Solver::new(500);

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle buffer"),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            size: std::mem::size_of_val(&*sph_solver.particles) as u64 * 3,
            mapped_at_creation: false,
        });

        Self {
            window_size,
            swap_chain,
            rt,
            sph_solver,
            pipeline,
            device,
            queue,
            particle_buffer,
        }
    }

    pub fn update(&mut self) {
        self.sph_solver.update();
    }

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });
        let tmp_buf = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Temp Buffer"),
            contents: bytemuck::cast_slice(&self.sph_solver.particles),
            usage: wgpu::BufferUsage::COPY_SRC,
        });

        let size = std::mem::size_of_val(&*self.sph_solver.particles);
        encoder.copy_buffer_to_buffer(&tmp_buf, 0, &self.particle_buffer, 0, size as u64);

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.particle_buffer.slice(..));
            render_pass.draw(0..self.sph_solver.particles.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
