use egui::epaint::ImageDelta;
use egui::ClippedPrimitive;
use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::InstanceDescriptor;
use egui_winit::egui;
use egui_winit::winit;

pub struct Window {
    winit_window: winit::window::Window,
    surface: wgpu::Surface,
    surface_conf: wgpu::SurfaceConfiguration,
    queue: wgpu::Queue,
    device: wgpu::Device,
    texture_fmt: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    renderer: egui_wgpu::Renderer,
    screen_descriptor: ScreenDescriptor,
    clear_colour: wgpu::Color,
}

impl Window {
    pub fn new<T>(event_loop: &winit::event_loop::EventLoop<T>) -> anyhow::Result<Self> {
        let winit_window = winit::window::Window::new(event_loop)?;
        let egui_wgpu::WgpuConfiguration {
            device_descriptor,
            backends,
            present_mode,
            power_preference,
            on_surface_error,
            depth_format,
        } = egui_wgpu::WgpuConfiguration::default();
        let instace = wgpu::Instance::new(InstanceDescriptor {
            backends,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });
        let surface = unsafe { instace.create_surface(&winit_window)? };
        let adapter = pollster::block_on(instace.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .ok_or(anyhow!("unable to create adapter"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&device_descriptor, None))?;
        let texture_fmt = wgpu::TextureFormat::Bgra8UnormSrgb;
        let present_mode = wgpu::PresentMode::AutoVsync;
        let window_size = winit_window.inner_size();
        let surface_conf = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_fmt,
            width: window_size.width,
            height: window_size.height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![texture_fmt],
        };
        surface.configure(&device, &surface_conf);
        let renderer = egui_wgpu::Renderer::new(&device, texture_fmt, None, 1);
        Ok(Self {
            winit_window,
            surface,
            surface_conf,
            queue,
            device,
            texture_fmt,
            present_mode,
            renderer,
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [window_size.width, window_size.height],
                pixels_per_point: 1.0,
            },
            clear_colour: wgpu::Color::BLACK,
        })
    }

    pub fn resized(&mut self) {
        let size = self.winit_window.inner_size();
        if size.width > 0 {
            self.surface_conf.width = size.width;
        }
        if size.height > 0 {
            self.surface_conf.height = size.height;
        }
        self.screen_descriptor.size_in_pixels = [self.surface_conf.width, self.surface_conf.height];
        self.surface.configure(&self.device, &self.surface_conf);
    }

    pub fn request_redraw(&self) {
        self.winit_window.request_redraw()
    }

    pub fn winit_window(&self) -> &winit::window::Window {
        &self.winit_window
    }

    pub fn render(
        &mut self,
        texture_set: Vec<(egui::TextureId, ImageDelta)>,
        texture_free: Vec<egui::TextureId>,
        clipped_primitives: &[ClippedPrimitive],
    ) {
        for (id, delta) in texture_set {
            self.renderer
                .update_texture(&self.device, &self.queue, id, &delta)
        }
        let frame = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(_) => {
                warn!("skipping frame");
                return;
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("texture view"),
            ..Default::default()
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            });
        let mut cmd_buffers = self.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            clipped_primitives,
            &self.screen_descriptor,
        );
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_colour),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.renderer
                .render(&mut rpass, &clipped_primitives, &self.screen_descriptor)
        }
        cmd_buffers.push(encoder.finish());
        self.queue.submit(cmd_buffers);
        frame.present();
        for text in texture_free {
            self.renderer.free_texture(&text);
        }
    }
}
