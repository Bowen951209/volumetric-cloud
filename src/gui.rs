use std::path::PathBuf;

use imgui::Condition;
use imgui_winit_support::HiDpiMode;
use winit::{event::Event, window::Window};

use crate::texture;

pub struct Gui {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

impl Gui {
    pub fn new(
        ini_filename: impl Into<Option<PathBuf>>,
        window: &Window,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let mut context = imgui::Context::create();
        context.set_ini_filename(ini_filename);

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), &window, HiDpiMode::Rounded);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: config.format,
            depth_format: Some(texture::DEPTH_FORMAT),
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(&mut context, &device, &queue, renderer_config);

        Self {
            context,
            platform,
            renderer,
        }
    }

    pub fn prepare_frame(&mut self, window: &Window) {
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare imgui frame");
    }

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
        self.platform
            .handle_event(self.context.io_mut(), window, event);
    }

    pub fn render_ui<'a>(
        &'a mut self,
        window: &Window,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let ui = self.context.frame();

        ui.window("Test")
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Test");
                ui.slider("no use slider", 10.0, 120.0, &mut 0.0);
            });

        self.platform.prepare_render(ui, window);
        let draw_data = self.context.render();
        self.renderer
            .render(draw_data, queue, &device, render_pass)
            .expect("Rendering failed");
    }
}
