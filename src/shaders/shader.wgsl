@vertex
fn vs_main(@location(0) pos: vec3f) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4f) -> @location(0) vec4f {
    return vec4f(1.0, 1.0, 1.0, 1.0);
}

