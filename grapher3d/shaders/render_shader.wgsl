const EPS: f32 = 0.01;
const MAX_STEPS: i32 = 100;
const SURFACE_DIST: f32 = 0.001;
const MAX_DIST: f32 = 100.0;

struct Camera {
    position: vec3<f32>,
    aspect_ratio: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// helper functions
// implicit sphere 
fn get_dist(p: vec3<f32>) -> f32 {
    let sphere_radius = 1.0;
    let sphere_centre = vec3<f32>(0.0, 0.0, 0.0);
    return length(p - sphere_centre) - sphere_radius; // for implicit function should be zero
}

// get normal for the Lambert diffusion
// normal vector is gradient of the implicit function 
fn calc_norm(p: vec3<f32>) -> vec3<f32> {
    let eps_vec = vec2<f32>(EPS, 0.0);
    let grad_impl = vec3<f32>(
        get_dist(p + eps_vec.xyy) - get_dist(p - eps_vec.xyy),
        get_dist(p + eps_vec.yxy) - get_dist(p - eps_vec.yxy),
        get_dist(p + eps_vec.yyx) - get_dist(p - eps_vec.yyx)
    );
    return normalize(grad_impl);
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    // Fullscreen triangle (oversized triangle)
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), // bottom-left
        vec2<f32>(3.0, -1.0), // bottom-right
        vec2<f32>(-1.0, 3.0), // top-left
    );

    let p = pos[vid];

    let uv = (p * 0.5) + vec2<f32>(0.5, 0.5);

    return VSOut(vec4<f32>(p, 0.0, 1.0), uv);
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let xy = (in.uv * 2.0 - 1.0) * vec2<f32>(camera.aspect_ratio, 1.0);

    let world_up = vec3<f32>(0.0, 1.0, 0.0);

    // camera vectors
    let camera_forward = normalize(-camera.position); // ||(0, 0, 0) - camera_camera||
    let camera_right = normalize(cross(camera_forward, world_up));
    let camera_up = cross(camera_right, camera_forward);

    let r_o = camera.position; // ray from the camera
    let r_d = normalize(camera_right * xy.x + camera_up * xy.y + camera_forward); // ray moves with camera

    // marching ray loop
    var d_o = 0.0; // distance from the origin
    var hit = false;

    for (var i = 0; i < MAX_STEPS; i++) {
        let p = r_o + r_d * d_o;
        let d_s = get_dist(p);
        d_o += d_s;

        if d_o > MAX_DIST || d_s < SURFACE_DIST {
            if d_s < SURFACE_DIST {
                hit = true;
                break;
            }
        }
    }

    // Lambert
    var color = vec3<f32>(0.05, 0.05, 0.1);

    if hit {
        let p = r_o + r_d * d_o;
        let n = calc_norm(p);

        // lambertian diffusion
        let light_postion = vec3<f32>(0.0, 0.0, 8.0);
        let l = normalize(light_postion - p);
        let diffuse = max(dot(n, l), 0.0);
        color = vec3<f32>(0.3, 0.7, 1.0) * diffuse;
        color += vec3<f32>(0.02);
    }

    return vec4<f32>(color, 1.0);
}