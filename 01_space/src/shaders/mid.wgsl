struct Uniforms {
    model_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = uniforms.projection_matrix * uniforms.model_matrix * vec4<f32>(in.position.xy, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    let colour = vec4<f32>(0.6, 0.2, 0.4, 1.0);

    return vec4(colour);
}

