const EPS: f32 = 0.1;
const MAX_STEPS: i32 = 256;
const SURFACE_DIST: f32 = 0.0001;
const MAX_DIST: f32 = 100.0;
const AXIS_THICKNESS: f32 = 0.1;
const WORLD_UP: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
const ZERO_VECTOR: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

struct Camera {
    position: vec3<f32>,
    aspect_ratio: f32,
    camera_pointing_at: vec3<f32>,
    _pad0: f32,
};

struct PlotConfig {
    min_bounds: vec3<f32>,
    _pad1: f32,
    max_bounds: vec3<f32>,
    _pad2: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var<uniform> plot_config: PlotConfig;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// helper functions
// get input
fn get_implicit_formula(p: vec3<f32>) -> f32 {
    return USER_INPUT;
}

// put boundaries by making a box and then use it to intersect with the shape
fn boundary_box(p: vec3<f32>, min_bounds: vec3<f32>, max_bounds: vec3<f32>) -> f32 {
    let half_length = (max_bounds - min_bounds) * 0.5;
    let center = (max_bounds + min_bounds) * 0.5;
    let q = abs(p - center) - half_length;
    return length(max(q, ZERO_VECTOR)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// draw axes
fn get_axes(p: vec3<f32>) -> f32 {
    // SDF for cylinder
    let x_axis = length(p.yz) - AXIS_THICKNESS;
    let z_axis = length(p.xy) - AXIS_THICKNESS;
    let y_axis = length(p.xz) - AXIS_THICKNESS;

    // combine all three
    return min(min(x_axis, y_axis), z_axis);
}

// intersection of the shape and the boundary box
// union between the axes and the clipped shape
fn get_dist(p: vec3<f32>) -> f32 {
    let axes = get_axes(p);
    let boundary_box = boundary_box(p, plot_config.min_bounds, plot_config.max_bounds);
    let shape = get_implicit_formula(p);
    // get the intersection
    let clipped_shape = max(boundary_box, shape);

    return min(axes, clipped_shape); // get the union
}

// get normal for the Lambert diffusion
fn calc_norm_coord(p: vec3<f32>) -> vec3<f32> {
    let eps_vec = vec2<f32>(EPS, 0.0);
    let grad_impl = vec3<f32>(
        get_implicit_formula(p + eps_vec.xyy) - get_implicit_formula(p - eps_vec.xyy),
        get_implicit_formula(p + eps_vec.yxy) - get_implicit_formula(p - eps_vec.yxy),
        get_implicit_formula(p + eps_vec.yyx) - get_implicit_formula(p - eps_vec.yyx)
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

    // camera vectors
    let camera_forward = normalize(camera.camera_pointing_at - camera.position);
    let camera_right = normalize(cross(camera_forward, WORLD_UP));
    let camera_up = cross(camera_right, camera_forward);

    // RayEquation : P(t) = r_o + d_o * r_d
    let r_o = camera.position; // ray from the camera origin
    let r_d = normalize(camera_right * xy.x + camera_up * xy.y + camera_forward);

    // marching ray loop
    var d_o = 0.0; // distance from the origin
    var hit = false;

    for (var i = 0; i < MAX_STEPS; i++) {
        let p = r_o + r_d * d_o;
        let d_s = get_dist(p);
        d_o += d_s * 0.5; // multiplied with a relaxation for a better convergence

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
        let n = calc_norm_coord(p);
        let light_pos = vec3<f32>(8.0, 8.0, 8.0);
        let l = normalize(light_pos - p);

        let diffuse = max(dot(n, l), 0.0);
        let ambient = 0.1;

        let object_color = vec4<f32>(0.8, 0.8, 0.9, 1.0);
        let final_rgb = object_color.rgb * (diffuse + ambient);

        return vec4<f32>(final_rgb, 1.0);
    }

    return vec4<f32>(color, 1.0);
}