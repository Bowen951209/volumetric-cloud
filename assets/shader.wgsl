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

    let color = raymarch(&ray, t_min, t_max, 0.1);
    return vec4<f32>(color, 1.0);
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

fn raymarch(ray: ptr<function, Ray>, t_min: f32, t_max: f32, step: f32) -> vec3<f32> {
    const LIGHT_POS = vec3<f32>(-2.0, 2.0, 2.0);
    const DENISTY = 0.8;

    var color_accum = vec3<f32>(0.0);
    var density_accum = 0.0;

    for(var t = t_min; t < t_max; t += step) {
        let pos = ray.origin + ray.direction * t;

        density_accum += DENISTY * (1.0 - density_accum);

        let distance_to_light = distance(pos, LIGHT_POS);
        let light_direction = (LIGHT_POS - pos) / distance_to_light;

        var light_amount = 1.0;
        for(var u = 0.0; u < distance_to_light; u += step) {
            let sample_pos = pos + light_direction * u;
            var temp1:f32;
            var temp2:f32;
            var aabb = aabb;
            var ray = Ray(sample_pos, light_direction);
            if (!intersect_aabb(&ray, &aabb, &temp1, &temp2)) {
                break;
            }

            light_amount *= exp(-DENISTY * step * 0.3);

            if (light_amount <= 0.01) {
                break;
            }
        }

        let contrib = DENISTY * (1.0 - density_accum);
        color_accum += vec3<f32>(light_amount) * contrib;
        density_accum += contrib;

        if(density_accum > 0.95) {
            break;
        }
    }

    return color_accum;
}