// Resolve coverage in linear light, then encode once for the compositor.
struct Uniforms { view_projection: mat4x4<f32>, selection: vec4<u32> }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var image: texture_2d<f32>;
@vertex fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2<f32>(-1., -1.), vec2<f32>(3., -1.), vec2<f32>(-1., 3.));
    return vec4<f32>(positions[index], 0., 1.);
}
@fragment fn fragment_main(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    let linear = textureLoad(image, vec2<i32>(at.xy), 0).rgb;
    let encoded = select(linear * 12.92, 1.055 * pow(linear, vec3<f32>(1. / 2.4)) - 0.055, linear > vec3<f32>(0.0031308));
    return vec4<f32>(select(linear, encoded, uniforms.selection.y != 0u), 1.);
}
