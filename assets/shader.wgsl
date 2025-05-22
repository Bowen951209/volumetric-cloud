struct CameraUniform {
    view_proj_inv: mat4x4<f32>,
    cam_pos: vec3<f32>,
};

struct AABBUniform {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
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

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var<uniform> aabb: AABBUniform;
@group(0) @binding(2)
var<uniform> screen_size: vec2<u32>;

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let ndc = vec2<f32>(
            (frag_coord.x / f32(screen_size.x)) * 2.0 - 1.0,
            1.0 - (frag_coord.y / f32(screen_size.y)) * 2.0
        );

    let clip = vec4<f32>(ndc, -1.0, 1.0);
    let world_pos = camera.view_proj_inv * clip;
    var ray = Ray (
        camera.cam_pos,
        normalize(world_pos.xyz / world_pos.w - camera.cam_pos),
    );

    var aabb = aabb;

    var t_min: f32;
    var t_max: f32;
    if (!intersect_aabb(&ray, &aabb, &t_min, &t_max)) {
        return vec4<f32>(0.0); // miss
    }

    let color = (t_max - t_min) / 10.0;
    return vec4<f32>(color, color, color, 1.0);
}

fn intersect_aabb(ray: ptr<function, Ray>,
                  box: ptr<function, AABBUniform>,
                  t_min_out: ptr<function, f32>, t_max_out: ptr<function, f32>) -> bool {
    var t_min = -1e10;
    var t_max = 1e10;

    for (var i = 0; i < 3; i++) {
        let inv_d = 1.0 / ray.direction[i];
        let t0 = (box.min[i] - ray.origin[i]) * inv_d;
        let t1 = (box.max[i] - ray.origin[i]) * inv_d;

        let t_near = min(t0, t1);
        let t_far = max(t0, t1);

        t_min = max(t_min, t_near);
        t_max = min(t_max, t_far);

        if (t_max < t_min) {
            return false;
        }
    }

    *t_min_out = t_min;
    *t_max_out = t_max;
    return true;
}
