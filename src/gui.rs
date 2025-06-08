use std::path::PathBuf;

use imgui::{Condition, Ui};
use imgui_winit_support::HiDpiMode;
use winit::{event::Event, window::Window};

use crate::{models::AABB, texture};

pub struct State {
    pub aabb: AABB,
    pub cloud_noise_size: u32,
    pub cloud_noise_size_power: u8,
    pub cloud_noise_frequency: f64,
    pub cloud_noise_seed: u32,
    pub should_create_new_cloud_noise: bool,
}

pub struct DisplayInfo {
    pub camera_position: [f32; 3],
    pub light_position: [f32; 3],
    pub fps: Option<f32>,
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
        info: &DisplayInfo,
    ) {
        let ui = self.context.frame();

        // Controls
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

        // Information
        ui.window("Information")
            .size([500.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                // FPS
                if let Some(fps) = info.fps {
                    ui.text(format!("FPS: {:.0}", fps));
                }

                // Camera position
                ui.text(format!(
                    "Camera Position: {}",
                    Gui::position_format(info.camera_position)
                ));

                // Light position
                ui.text(format!(
                    "Light Position: {}",
                    Gui::position_format(info.light_position)
                ));
            });

        // Cloud noise
        ui.window("Cloud Noise")
            .size([300.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                // size power
                ui.slider("size power", 6, 16, &mut self.state.cloud_noise_size_power);
                self.state.cloud_noise_size = 1u32 << self.state.cloud_noise_size_power;
                ui.text(format!("Texture size: {}", self.state.cloud_noise_size));

                // frequency
                ui.slider(
                    "frequency",
                    0.001,
                    1.0,
                    &mut self.state.cloud_noise_frequency,
                );

                // seed
                ui.input_scalar("seed", &mut self.state.cloud_noise_seed)
                    .build();

                // generate button
                self.state.should_create_new_cloud_noise = ui.button("Generate");
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

    fn position_format(position: [f32; 3]) -> String {
        format!(
            "({:.3}, {:.3}, {:.3})",
            position[0], position[1], position[2]
        )
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
