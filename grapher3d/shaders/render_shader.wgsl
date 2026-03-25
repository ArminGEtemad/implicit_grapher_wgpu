const EPS: f32 = 0.001;
const MAX_STEPS: i32 = 128;
const SURFACE_DIST: f32 = 0.001;
const MAX_DIST: f32 = 100.0;
const AXIS_THICKNESS: f32 = 0.01;

struct Camera {
    position: vec3<f32>,
    aspect_ratio: f32,
    camera_pointing_at: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// helper functions
// get input
fn get_implicit_formula(p: vec3<f32>) -> f32 {
    return USER_INPUT;
}

// 3D coordinates
fn make_coordinate(p: vec3<f32>) -> f32 {
    // make three infinite cylinders
    let x = length(p.yz) - AXIS_THICKNESS;
    let y = length(p.xz) - AXIS_THICKNESS;
    let z = length(p.xy) - AXIS_THICKNESS;

    let axes = min(x, min(y, z));
    return axes;
}
fn get_dist(p: vec3<f32>) -> f32 {
    let axes = make_coordinate(p);
    let graph = get_implicit_formula(p);
    return min(axes, graph);
}

// get normal for the Lambert diffusion
// normal vector is gradient of the implicit function 
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

    let world_up = vec3<f32>(0.0, 1.0, 0.0);

    // camera vectors
    let camera_forward = normalize(camera.camera_pointing_at - camera.position);
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
        let n = calc_norm_coord(p);

        // distances at the hit point to see "who" is hit
        let d_axes = make_coordinate(p);
        let d_graph = get_implicit_formula(p);

        var base_color = vec3<f32>(0.0);

        // Check if we hit the Graph first
        if d_graph < d_axes {
            // shape 
            base_color = vec3<f32>(0.8, 0.8, 0.9);
        } else {
            // hit an axis, now determine which one
            let x_dist = length(p.yz);
            let y_dist = length(p.xz);
            let z_dist = length(p.xy);

            if x_dist < y_dist && x_dist < z_dist {
                base_color = vec3<f32>(1.0, 0.1, 0.1); // Red X
            } else if y_dist < z_dist {
                base_color = vec3<f32>(0.1, 1.0, 0.1); // Green Y
            } else {
                base_color = vec3<f32>(0.1, 0.1, 1.0); // Blue Z
            }
        }

        // Lighting
        let light_position = vec3<f32>(8.0, 8.0, 8.0);
        let l = normalize(light_position - p);
        let diffuse = max(dot(n, l), 0.0);

        let final_color = base_color * diffuse + 0.01;
        return vec4<f32>(final_color, 1.0);
    }

    return vec4<f32>(color, 1.0);
}