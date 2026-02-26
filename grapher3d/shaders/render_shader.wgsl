struct VSOut {
    @builtin(position) pos : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    // Fullscreen triangle (oversized triangle)
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), // bottom-left
        vec2<f32>( 3.0,  -1.0), // bottom-right
        vec2<f32>(-1.0,  3.0), // top-left
    );

    let p = pos[vid];

    let uv = (p * 0.5) + vec2<f32>(0.5, 0.5);

    return VSOut(vec4<f32>(p, 0.0, 1.0), uv);
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let color = vec3<f32>(1.0 - in.uv.x, in.uv.y, in.uv.y);
    return vec4<f32>(color, 1.0); 
}