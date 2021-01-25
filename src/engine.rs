use shaderc::ShaderKind;
use wgpu::{BindGroupLayoutDescriptor, PipelineLayoutDescriptor, RenderPipelineDescriptor};

use crate::solver::Solver;

pub struct Engine {
    window_size: winit::dpi::PhysicalSize<u32>,
    swap_chain: wgpu::SwapChain,
    rt: tokio::runtime::Runtime,
    sph_solver: Solver,
    pipeline: wgpu::RenderPipeline,
}

impl Engine {
    pub async fn new(window: &winit::window::Window) -> Self {
        let window_size = window.inner_size();
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
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
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let mut compiler = shaderc::Compiler::new().unwrap();
        let spirv = compiler
            .compile_into_spirv(
                include_str!("shader/shader.vert"),
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();

        let vertex_stage =
            device.create_shader_module(wgpu::util::make_spirv(spirv.as_binary_u8()));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vertex_stage,
                entry_point: "main",
            },
            fragment_stage: None,
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::PointList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .worker_threads(4)
            .build()
            .unwrap();

        let sph_solver = Solver::new(500, 16.0);

        Self {
            window_size,
            swap_chain,
            rt,
            sph_solver,
            pipeline,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
    }
}
