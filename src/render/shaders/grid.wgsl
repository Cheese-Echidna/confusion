struct Uniforms {view_projection:mat4x4<f32>,selection:vec4<u32>}
@group(0) @binding(0) var<uniform> uniforms:Uniforms;
struct Output {@builtin(position) position:vec4<f32>,@location(0) color:vec3<f32>}
@vertex fn vertex_main(@location(0) position:vec3<f32>,@location(1) color:vec3<f32>)->Output{
    var output:Output;output.position=uniforms.view_projection*vec4<f32>(position,1.0);output.color=color;return output;
}
struct Fragment {@location(0) color:vec4<f32>,@location(1) face:u32}
@fragment fn fragment_main(input:Output)->Fragment{
    let linear=input.color;
    let encoded=select(linear*12.92,1.055*pow(linear,vec3<f32>(1.0/2.4))-0.055,linear>vec3<f32>(0.0031308));
    var result:Fragment;result.color=vec4<f32>(select(linear,encoded,uniforms.selection.y!=0u),1.0);result.face=0u;return result;
}
