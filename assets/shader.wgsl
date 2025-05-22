struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    switch(vertex_index) {
        case 0: {out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0); break;}
        case 1: {out.clip_position = vec4<f32>(1.0, -1.0, 0.0, 1.0); break;}
        case 2: {out.clip_position = vec4<f32>(1.0, 1.0, 0.0, 1.0); break;}
        case 3: {out.clip_position = vec4<f32>(1.0, 1.0, 0.0, 1.0); break;}
        case 4: {out.clip_position = vec4<f32>(-1.0, 1.0, 0.0, 1.0); break;}
        case 5: {out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0); break;}
        default: {out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 1.0); break;}
    }

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}