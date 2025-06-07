use std::path::PathBuf;

use imgui::{Condition, Ui};
use imgui_winit_support::HiDpiMode;
use winit::{event::Event, window::Window};

use crate::{models::AABB, texture};

pub struct State {
    pub aabb: AABB,
}

pub struct Gui {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    state: State,
}

impl Gui {
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn new(
        ini_filename: impl Into<Option<PathBuf>>,
        window: &Window,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        state: State,
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
            state,
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

        ui.window("Controls")
            .size([300.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                const XYZ_MIN: [f32; 3] = [-5.0; 3];
                const XYZ_MAX: [f32; 3] = [5.0; 3];
                let aabb = &mut self.state.aabb;

                ui.text("AABB Min");
                ui.slider_float3(
                    &["min x", "min y", "min z"],
                    &mut aabb.min,
                    &XYZ_MIN,
                    &aabb.max,
                );

                ui.text("AABB Max");
                ui.slider_float3(
                    &["max x", "max y", "max z"],
                    &mut aabb.max,
                    &aabb.min,
                    &XYZ_MAX,
                );
            });

        self.platform.prepare_render(ui, window);
        let draw_data = self.context.render();
        self.renderer
            .render(draw_data, queue, &device, render_pass)
            .expect("Rendering failed");
    }

    pub fn want_capture_window_event(&self) -> bool {
        let io = self.context.io();
        io.want_capture_keyboard || io.want_capture_mouse
    }
}

pub trait UiExtension {
    fn slider_float3(
        &self,
        labels: &[&str; 3],
        values: &mut [f32; 3],
        mins: &[f32; 3],
        maxs: &[f32; 3],
    ) -> bool;
}

impl UiExtension for Ui {
    fn slider_float3(
        &self,
        labels: &[&str; 3],
        values: &mut [f32; 3],
        mins: &[f32; 3],
        maxs: &[f32; 3],
    ) -> bool {
        let mut is_edited = false;
        for i in 0..3 {
            self.slider(labels[i], mins[i], maxs[i], &mut values[i])
                .then(|| {
                    is_edited = true;
                });

            if values[i] < mins[i] {
                values[i] = mins[i];
            } else if values[i] > maxs[i] {
                values[i] = maxs[i];
            }
        }

        is_edited
    }
}
