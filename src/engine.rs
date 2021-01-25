pub struct Engine {
    window_size: winit::dpi::PhysicalSize<u32>,
    swap_chain: wgpu::SwapChain,
    rt: tokio::runtime::Runtime,
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

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .worker_threads(4)
            .build()
            .unwrap();

        Self {
            window_size,
            swap_chain,
            rt,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
    }
}
