use std::sync::Arc;

use wgpu;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct Game {
    // Surface has a lifetime parameter because it owns the Window.
    // By putting the Arc<>, it can be cloned and passed to wgpu::Instance::create_surface().
    // This ensures that the window esists for as long as the surface does.
    window: Option<Arc<winit::window::Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("pawo the game")
                .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0));

            // By putting the Arc<>, it can be cloned and passed to wgpu::Instance::create_surface().
            // This ensures that the window esists for as long as the surface does.
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            self.window = Some(window.clone());
            self.size = window.inner_size();

            // The instance is a handle to the GPU.
            // It's main purpose is to create Surfaces and Adapters.
            let instance = wgpu::Instance::default();

            self.surface = Some(instance.create_surface(window).unwrap());

            pollster::block_on(self.init_gfx(&instance));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("{event:?}");

        match event {
            WindowEvent::CloseRequested => {
                println!("Close butten pressed, stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }

            // Catch all other unimplemented states.
            _ => {
                println!("oeps");
            }
        }
    }
}

impl Game {
    async fn init_gfx(&mut self, instance: &wgpu::Instance) {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface.
                compatible_surface: self.surface.as_ref(),
            })
            .await
            .expect("Failed to find an apropriate adapter.");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter,
                    // so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = self
            .surface
            .as_ref()
            .expect("no surface")
            .get_capabilities(&adapter);

        // Assume we use sRGB format for shaders.
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        // The configuration for the surface defines how the underlying SurfaceTexture is created.
        let config = self
            .surface
            .as_ref()
            .expect("no surface")
            .get_default_config(&adapter, self.size.width, self.size.height)
            .unwrap();

        self.surface
            .as_ref()
            .expect("no surface")
            .configure(&device, &config);

        self.config = Some(config.clone());
        self.device = Some(device);
        self.queue = Some(queue);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.as_mut().expect("no config").width = new_size.width;
            self.config.as_mut().expect("no config").height = new_size.height;
            self.surface.as_ref().expect("no surface").configure(
                &self.device.as_ref().expect("no device"),
                &self.config.as_ref().expect("no device"),
            );
        }
    }
}

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    // event_loop.set_control_flow(ControlFlow::Poll);

    env_logger::init();
    let mut game = Game::default();

    //let game = pollster::block_on(Game::new(event_loop)));

    event_loop.run_app(&mut game)
}
