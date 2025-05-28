use noise::{NoiseFn, Worley};
use wgpu::{TextureFormat, TextureView};

pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

pub fn create_noise_texture_3d(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    size: wgpu::Extent3d,
    label: Option<&str>,
    seed: u32,
    frequency: f64,
) -> wgpu::Texture {
    let generator = Worley::new(seed).set_frequency(frequency);

    let mut data = vec![0.0; (size.width * size.height * size.depth_or_array_layers) as usize];

    for z in 0..size.depth_or_array_layers {
        for y in 0..size.height {
            for x in 0..size.width {
                // Get noise in range [-1, 1], and map to [0, 1]
                let noise = (generator.get([x as f64, y as f64, z as f64]) as f32) * 0.5 + 0.5;
                data[(z * size.width * size.height + y * size.width + x) as usize] = noise;
            }
        }
    }

    create_texture_3d_gray(device, queue, size, &data, label)
}

pub fn create_texture_3d_gray(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    size: wgpu::Extent3d,
    data: &[f32],
    label: Option<&str>,
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(data),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size.width * std::mem::size_of::<f32>() as u32),
            rows_per_image: Some(size.height),
        },
        size,
    );

    texture
}

pub fn create_depth_texture_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    label: &str,
) -> TextureView {
    let size = wgpu::Extent3d {
        width: config.width.max(1),
        height: config.height.max(1),
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let texture = device.create_texture(&desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    view
}
