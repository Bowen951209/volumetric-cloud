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
@group(0) @binding(3)
var<uniform> light_pos: vec3<f32>;

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


    if(intersect_sphere(ray, light_pos, 0.3)) {
        return vec4(1.0);
    }

    var t_min: f32;
    var t_max: f32;
    if (!intersect_aabb(ray, aabb, &t_min, &t_max)) {
        return vec4<f32>(0.0); // miss
    }


    let color = raymarch_in_box(ray, t_min, t_max, 0.1);
    return color;
}

fn intersect_aabb(ray: Ray,
                  box: AABBUniform,
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

fn intersect_sphere(ray: Ray, sphere_center: vec3<f32>, radius: f32) -> bool {
    let oc = ray.origin - sphere_center;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    return discriminant > 0.0;
}

const DENISTY = 0.8;

fn raymarch_in_box(ray: Ray, t_min: f32, t_max: f32, step: f32) -> vec4<f32> {
    var color = vec3<f32>(0.0);
    var transmittance = 1.0;

    for(var t = t_min; t < t_max; t += step) {
        let pos = ray.origin + ray.direction * t;
        let density = DENISTY;

        if(density < 0.01) {
            continue;
        }

        let distance_to_light = distance(pos, light_pos);
        let ray_to_light = Ray(pos, (light_pos - pos) / distance_to_light);
        let light = raymarch_to_light(ray_to_light, 0.1);
        let phase = compute_phase(dot(ray.direction, ray_to_light.direction));
        let scattered = vec3<f32>(density * light * phase);

        color += scattered * transmittance;

        transmittance *= exp(-density * 0.5 * step);
        if (transmittance < 0.01) {
            break;
        }
    }

    
    return vec4<f32>(vec3<f32>(color), 1.0 - transmittance);
}


fn raymarch_to_light(ray: Ray, step: f32) -> f32 {
    var t_min: f32;
    var t_max: f32;
    intersect_aabb(ray, aabb, &t_min, &t_max);

    var transmittance = 1.0;
    for(var t = 0.0; t < t_max; t += step) {
        let pos = ray.origin + ray.direction * t;
        let density = DENISTY;
        transmittance *= exp(-density * 0.5 * step);
        if (transmittance < 0.01) {
            break;
        }
    }

    return transmittance;
}

const PI = radians(180.0);
fn compute_phase(cosTheta: f32) -> f32 {
    let g = 0.6;
    let denom = 1.0 + g * g - 2.0 * g * cosTheta;
    return (1.0 - g * g) / (4.0 * PI * pow(denom, 1.5));
}