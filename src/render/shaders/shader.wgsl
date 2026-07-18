struct Uniforms {
    model_mat: mat4x4<f32>,
    view_project_mat: mat4x4<f32>,
};
@binding(0) @group(0) var<uniform> uniforms : Uniforms;

struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) v_position: vec4<f32>,
    @location(1) v_color: vec4<f32>,
    @location(2) v_barycentric: vec4<f32>,
    @location(3) v_sides: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) barycentric: vec4<f32>,
    @location(3) sides: vec4<f32>,
) -> Output {
    var output: Output;
    let m_position: vec4<f32> = uniforms.model_mat * position;

    output.v_position = m_position;
    output.v_color = color;
    output.v_barycentric = barycentric;
    output.v_sides = sides;

    output.position = uniforms.view_project_mat * m_position;
    return output;
}

struct FragUniforms {
    line_thickness: f32,
	line_mode: f32,
	_padding: vec2<f32>,
};
@binding(1) @group(0) var<uniform> frag_uniforms : FragUniforms;

fn edge(v_barycentric: vec3<f32>, v_sides: vec3<f32>) -> bool {
    let face: vec3<f32> = v_barycentric * v_sides;
    let r: vec3<f32> = fwidthFine(face) * frag_uniforms.line_thickness;
    let f: vec3<f32> = step(r, face);
    return min(min(f.x, f.y), f.z) == 0.0;
}

fn vertex(v_barycentric: vec3<f32>, v_sides: vec3<f32>) -> vec3<f32> {
    let face: vec3<f32> = v_barycentric * v_sides;
    let r: vec3<f32> = fwidthFine(face) * frag_uniforms.line_thickness;
    let f: vec3<f32> = step(r, face);
    return step(v_barycentric, v_sides);
}

@fragment
fn fs_main(
    @location(0) v_position: vec4<f32>,
    @location(1) v_color: vec4<f32>,
    @location(2) v_barycentric: vec4<f32>,
    @location(3) v_sides: vec4<f32>,
) -> @location(0) vec4<f32> {
    if frag_uniforms.line_mode != 0.0 {
        if edge(v_barycentric.xyz, v_sides.xyz) {
            return vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            return v_color;
        }
    } else {
        if edge(v_barycentric.xyz, v_sides.xyz) {
            return v_color;
        } else {
            return vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
}
