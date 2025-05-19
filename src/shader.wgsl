struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) vert_color: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;

    switch(in_vertex_index) {
        case 0: {
            out.vert_color = vec3<f32>(0.0, 1.0, 0.0);
            break;
        }
        case 1: {
            out.vert_color = vec3<f32>(1.0, 0.0, 0.0);
            break;
        }
        case 2: {
            out.vert_color = vec3<f32>(0.0, 0.0, 1.0);
            break;
        }
        default : {
            out.vert_color = vec3<f32>(0.0, 0.0, 0.0);
        }
    }

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.vert_color, 1.0);
}
 