const EPS: f32 = 0.001;
const MAX_STEPS: i32 = 256;
const SURFACE_DIST: f32 = 0.0001;
const MAX_DIST: f32 = 128.0;
const AXIS_THICKNESS: f32 = 0.1;
const WORLD_UP: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
const ZERO_VECTOR: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const BACKGROUND_COLOR: vec3<f32> = vec3<f32>(0.05, 0.05, 0.1);

// for now hard coded
const HEIGHT: f32 = 800.0;
const WIDTH: f32 = 1200.0;

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

struct DynamicScene {
    time: f32,
    frame_count: u32,
    _pad0: vec2<f32>,
};

struct RayMarcherOutput {
    color: vec3<f32>,
    depth: f32,
    surface_normal: vec3<f32>,
    hit: f32,
};

struct AxisDesc {
    dist: f32,
    normal: vec3<f32>,
};

struct ImplicitShapeDesc {
    dist: f32,
    normal: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var<uniform> plot_config: PlotConfig;

@group(0) @binding(2)
var<uniform> scene: DynamicScene;

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
fn get_axes(p: vec3<f32>) -> AxisDesc {
    // SDF for cylinder
    let x_axis = length(p.yz) - AXIS_THICKNESS;
    let z_axis = length(p.xy) - AXIS_THICKNESS;
    let y_axis = length(p.xz) - AXIS_THICKNESS;

    var closest = x_axis;
    var normal = normalize(vec3<f32>(0.0, p.y, p.z));

    if y_axis < closest {
        closest = y_axis;
        normal = normalize(vec3<f32>(p.x, 0.0, p.z));
    }

    if z_axis < closest {
        closest = z_axis;
        normal = normalize(vec3<f32>(p.x, p.y, 0.0));
    }

    return AxisDesc(closest, normal);
}

// gradient calculation for an implicit surface 
// results in the normal vector
fn grad_calc(p: vec3<f32>) -> vec3<f32> {
    let eps_vec = vec2<f32>(EPS, 0.0);
    let grad_vec = vec3<f32>(
        get_implicit_formula(p + eps_vec.xyy) - get_implicit_formula(p - eps_vec.xyy),
        get_implicit_formula(p + eps_vec.yxy) - get_implicit_formula(p - eps_vec.yxy),
        get_implicit_formula(p + eps_vec.yyx) - get_implicit_formula(p - eps_vec.yyx)
    ) / (2.0 * EPS);

    return grad_vec;
}

fn get_hart_shape(p: vec3<f32>) -> ImplicitShapeDesc {
    let box_dist = boundary_box(p, plot_config.min_bounds, plot_config.max_bounds);

    // intersection of the shape and the boundary box
    // Calculate the "safe" distance for the implicit shape
    let f = get_implicit_formula(p);
    let g = grad_calc(p);
    let clipped_shape_dist = max(box_dist, abs(f) / max(length(g), 0.0001));
    let n = g / max(length(g), 0.0001);
    return ImplicitShapeDesc(clipped_shape_dist, n);
}

// union between the axes and the clipped shape
fn shape_axes_combination(p: vec3<f32>) -> f32 {
    let axes_dist = get_axes(p).dist;
    let shape_dist = get_hart_shape(p).dist;

    return min(axes_dist, shape_dist);
}

// raymarching logic
fn raymarcher(uv: vec2<f32>) -> RayMarcherOutput {
    let xy = (uv * 2.0 - 1.0) * vec2<f32>(camera.aspect_ratio, 1.0);

    // define camera directions
    let camera_forward = normalize(camera.camera_pointing_at - camera.position);
    let camera_right = normalize(cross(camera_forward, WORLD_UP));
    let camera_up = cross(camera_right, camera_forward);

    // ray equation p(t) = r_o + d_o * r_d
    let r_o = camera.position;
    let r_d = normalize(camera_right * xy.x + camera_up * xy.y + camera_forward);

    // marching loop
    var d_o = 0.0;
    var hit = false;

    for (var i = 0; i < MAX_STEPS; i++) {
        let p = r_o + d_o * r_d;
        let d_s = shape_axes_combination(p) * 0.5;
        d_o += d_s;

        if d_o > MAX_DIST || d_s < SURFACE_DIST {
            if d_s < SURFACE_DIST {
                hit = true;
                break;
            }
        }
    }

    // Lambert
    var color = BACKGROUND_COLOR;
    if hit {
        let p = r_o + r_d * d_o;

        let axes = get_axes(p);
        let shape = get_hart_shape(p);

        var n = ZERO_VECTOR;
        if axes.dist < shape.dist {
            n = axes.normal;
        } else {
            n = shape.normal;
        }

        let light_pos = vec3<f32>(1.0, 10.0, 3.0);
        let l = normalize(light_pos - p);

        let diffuse = max(dot(n, l), 0.0);
        let ambient = 0.1;

        let object_color = vec4<f32>(0.8, 0.8, 0.9, 1.0);
        let final_rgb = object_color.rgb * (diffuse + ambient);

        return RayMarcherOutput(final_rgb, d_o, n, 1.0);
    }

    return RayMarcherOutput(BACKGROUND_COLOR, MAX_DIST, vec3<f32>(0.0), 0.0);
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
    let raymarcher_result = raymarcher(in.uv);

    // edge detection
    let depth_delta = fwidth(raymarcher_result.depth) / (raymarcher_result.depth + 0.1);
    let normal_delta = length(fwidth(raymarcher_result.surface_normal));
    let hit_delta = fwidth(raymarcher_result.hit);

    // is it the edge?
    let is_edge = hit_delta > 0.0 || depth_delta > 0.05 || normal_delta > 0.1;

    // oversampling for the edge only
    let grid_offset = array<vec2<f32>, 4>(
        vec2<f32>(-0.25, -0.25),
        vec2<f32>(0.25, -0.25),
        vec2<f32>(-0.25, 0.25),
        vec2<f32>(0.25, 0.25),
    );

    var oversampled_raymarcher_color = vec3<f32>(0.0);
    var color = raymarcher_result.color;
    if is_edge {
        let dx = 1.0 / WIDTH;
        let dy = 1.0 / HEIGHT;

        for (var i = 0; i < 4; i++) {
            let sub_xy = in.uv + vec2<f32>(grid_offset[i].x * dx, grid_offset[i].y * dy);

            color += raymarcher(sub_xy).color;
        }
        oversampled_raymarcher_color = color * 0.2; // averaged color after oversampling
    } else {
        oversampled_raymarcher_color = raymarcher_result.color;
    }

    return vec4<f32>(oversampled_raymarcher_color, 1.0);
}