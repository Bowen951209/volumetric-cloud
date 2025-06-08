use std::path::Path;

use noise::{NoiseFn, Worley};

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub fn load_texture_2d_gray<P: AsRef<Path>>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &P,
) -> image::ImageResult<wgpu::Texture> {
    let image = image::open(path)?.to_luma32f();
    let (width, height) = image.dimensions();

    // Get data of &[u8]
    let raw_data = image.into_raw();
    let raw_data = bytemuck::cast_slice(&raw_data);

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let file_name = path
        .as_ref()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(format!("Luma32f (Gray) Texture {}", file_name.as_str()).as_str()),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfoBase {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &raw_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * std::mem::size_of::<f32>() as u32),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    Ok(texture)
}

pub fn create_noise_texture_3d(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    size: wgpu::Extent3d,
    label: Option<&str>,
    seed: u32,
    frequency: f64,
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

    write_noise_texture_3d(queue, &texture, seed, frequency);
    texture
}

pub fn create_depth_texture_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    label: &str,
) -> wgpu::TextureView {
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

fn write_noise_texture_3d(queue: &wgpu::Queue, texture: &wgpu::Texture, seed: u32, frequency: f64) {
    let size = texture.size();
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

    write_texture_3d(queue, &texture, bytemuck::cast_slice(&data));
}

fn write_texture_3d(queue: &wgpu::Queue, texture: &wgpu::Texture, data: &[u8]) {
    let size = texture.size();
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size.width * std::mem::size_of::<f32>() as u32),
            rows_per_image: Some(size.height),
        },
        size,
    );
}
