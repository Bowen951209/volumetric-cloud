#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct AABB {
    pub min: [f32; 3],
    _padding1: f32,
    pub max: [f32; 3],
    _padding2: f32,
}

impl AABB {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self {
            min,
            max,
            ..Default::default()
        }
    }
}
