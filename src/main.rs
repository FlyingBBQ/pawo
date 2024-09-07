use std::sync::Arc;

use wgpu;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[derive(Default)]
struct Game {
    // Surface has a lifetime parameter because it owns the Window.
    // By putting the Arc<>, it can be cloned and passed to wgpu::Instance::create_surface().
    // This ensures that the window esists for as long as the surface does.
    window: Option<Arc<winit::window::Window>>,
    gpu: Option<Gpu<'static>>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("pawo the game")
            .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0));

        // By putting the Arc<>, it can be cloned and passed to wgpu::Instance::create_surface().
        // This ensures that the window esists for as long as the surface does.
        if let Ok(window) = event_loop.create_window(window_attributes) {
            let first_window_handle = self.window.is_none();

            let window_handle = Arc::new(window);
            self.window = Some(window_handle.clone());

            if first_window_handle {
                self.size = window_handle.inner_size();

                let (width, height) = (
                    window_handle.inner_size().width,
                    window_handle.inner_size().height,
                );

                let gpu = pollster::block_on(async move {
                    Gpu::new_async(window_handle.clone(), width, height).await
                });
                self.gpu = Some(gpu);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("{event:?}");

        let Some(gpu) = self.gpu.as_mut() else { return; };

        match event {
            WindowEvent::CloseRequested => {
                println!("Close butten pressed, stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                log::info!("Resizing surface to: ({width}, {height})");
                gpu.resize(width, height);
            }

            // Catch all other unimplemented states.
            _ => {
                println!("oeps");
            }
        }
    }
}

struct Gpu<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
}

impl<'window> Gpu<'window> {
    async fn new_async(
        window: impl Into<wgpu::SurfaceTarget<'window>>,
        width: u32,
        height: u32,
    ) -> Self {
        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface.
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an apropriate adapter.");

        log::info!("WGPU Adapter Features: {:#?}", adapter.features());

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WGPU device"),
                    required_features: wgpu::Features::default(),
                    // Make sure we use the texture resolution limits from the adapter,
                    // so we can support images the size of the surface.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Assume we use sRGB format for shaders.
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        // The configuration for the surface defines how the underlying SurfaceTexture is created.
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Self {
            surface,
            device,
            queue,
            surface_config,
            surface_format,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    env_logger::init();
    let mut game = Game::default();

    //let game = pollster::block_on(Game::new(event_loop)));

    event_loop.run_app(&mut game)
}
