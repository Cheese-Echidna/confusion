struct Uniforms {
    view_projection: mat4x4<f32>,
    selection: vec4<u32>,
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) local_position: vec3<f32>,
    @location(2) @interpolate(flat) face: u32,
}
@vertex
fn vertex_main(@location(0) position: vec3<f32>,
               @location(1) normal: vec3<f32>,
               @location(2) face: u32) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.view_projection * vec4<f32>(position, 1.0);
    output.normal = normal;
    output.local_position = position;
    output.face = face;
    return output;
}
struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) face: u32,
}
fn fragment_main(input: VertexOutput) -> FragmentOutput {
    let normal = normalize(input.normal);
    let diffuse = max(dot(normal, normalize(vec3<f32>(0.4, -0.5, 1.0))), 0.0);
    let base = select(vec3<f32>(0.227, 0.376, 0.314), vec3<f32>(0.948, 0.508, 0.028),
                      input.face == uniforms.selection.x);
    var output: FragmentOutput;
    let linear = base * (0.38 + diffuse * 0.62);
    // Encode only after multisample coverage is resolved in linear light.
    output.color = vec4<f32>(linear, 1.0);
    output.face = input.face;
    return output;
}

@fragment
fn fragment_color(input: VertexOutput) -> @location(0) vec4<f32> {
    return fragment_main(input).color;
}
@fragment
fn fragment_pick(input: VertexOutput) -> @location(0) u32 {
    return input.face;
}
