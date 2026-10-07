struct Uniforms {
    view_projection: mat4x4<f32>,
    selection: vec4<u32>,
    eye: vec4<f32>,
    outward: vec4<f32>,
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) visible: u32,
}
@vertex fn vertex_main(@location(0) position: vec3<f32>, @location(1) first: vec3<f32>,
                      @location(2) second: vec3<f32>, @location(3) boundary: u32) -> Output {
    var out: Output;
    out.position = uniforms.view_projection * vec4<f32>(position, 1.0);
    out.position.z -= 0.000005*out.position.w;
    let view = select(uniforms.outward.xyz, uniforms.eye.xyz-position, uniforms.eye.w>0.5);
    out.visible = select(0u, 1u, boundary != 0u || dot(first,view)*dot(second,view)<=0.0);
    return out;
}
@fragment fn visible(input: Output) -> @location(0) vec4<f32> {
    if(input.visible==0u){discard;}
    return vec4<f32>(select(vec3<f32>(0.008,0.01,0.014),vec3<f32>(0.55,0.61,0.69),uniforms.selection.z!=0u),1.0);
}
@fragment fn hidden(input: Output) -> @location(0) vec4<f32> {
    if(input.visible==0u || (u32(input.position.x+input.position.y)/5u)%2u==0u){discard;}
    return vec4<f32>(0.21,0.23,0.26,1.0);
}
@fragment fn wire(input: Output) -> @location(0) vec4<f32> {
    if(input.visible==0u){discard;}
    return vec4<f32>(0.55,0.61,0.69,1.0);
}
