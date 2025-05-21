use cgmath::Vector2;
use wgpu::{
    Device, RenderPipeline, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_elements: u32,
}

impl Mesh {
    pub fn create_quad(
        device: &Device,
        min: Vector2<f32>,
        max: Vector2<f32>,
        label: Option<&str>,
    ) -> Self {
        let vertices = [
            [min.x, min.y, 0.0], // bottom left
            [max.x, min.y, 0.0], // bottom right
            [max.x, max.y, 0.0], // top right
            [min.x, max.y, 0.0], // top left
        ];

        const INDICES: [usize; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: label.map(|s| format!("{s} vertex buffer")).as_deref(),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: label.map(|s| format!("{s} index buffer")).as_deref(),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_elements: INDICES.len() as u32,
        }
    }

    pub fn create_full_screen_quad(device: &Device, label: Option<&str>) -> Self {
        Self::create_quad(
            device,
            Vector2::new(-1.0, -1.0),
            Vector2::new(1.0, 1.0),
            label,
        )
    }

    pub fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 3]>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, pipeline: &RenderPipeline, mesh: &Mesh);
}

impl<'a> DrawModel<'a> for wgpu::RenderPass<'a> {
    fn draw_mesh(&mut self, pipeline: &RenderPipeline, mesh: &Mesh) {
        self.set_pipeline(pipeline);
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }
}
