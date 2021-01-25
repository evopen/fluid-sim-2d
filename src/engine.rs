use vk_shader_macros::include_glsl;
use wgpu::{BindGroupLayoutDescriptor, PipelineLayoutDescriptor, RenderPipelineDescriptor};

use crate::solver::Solver;

pub struct Engine {
    window_size: winit::dpi::PhysicalSize<u32>,
    swap_chain: wgpu::SwapChain,
    rt: tokio::runtime::Runtime,
    sph_solver: Solver,
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

        let vertex_shader = include_glsl!(tokens)

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
            layout: None,
            vertex_stage: (),
            fragment_stage: None,
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::PointList,
            color_states: wgpu::ColorStateDescriptor{
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                alpha_blend: ,
                color_blend: (),
                write_mask: (),
                
            },
            depth_stencil_state: (),
            vertex_state: wgpu::VertexStateDescriptor{
                index_format: (),
                vertex_buffers: (),
                
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
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
    }
}
