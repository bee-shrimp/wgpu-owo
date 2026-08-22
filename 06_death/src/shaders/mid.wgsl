struct Uniforms {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var diffuse_texture: texture_2d<f32>;
@group(0) @binding(2)
var sampler_nearest: sampler;

struct VertexInput {
    @location(0) v_pos: vec2<f32>, // vertex buffer position
    @location(1) uv: vec2<f32>,
    @location(2) i_pos: vec2<f32>, // instance buffer position
    @location(3) size: vec2<f32>,
    @location(4) sprite_offset: vec2<f32>,
    @location(5) sprite_size: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    let pos = in.i_pos + in.v_pos * in.size;

    out.position = uniforms.projection_matrix * uniforms.view_matrix * vec4<f32>(pos, 0.0, 1.0);

    out.uv = in.sprite_offset + in.v_pos * in.sprite_size;
    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    //let colour = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let colour = textureSample(diffuse_texture, sampler_nearest, uv);

    return vec4(colour);
}

