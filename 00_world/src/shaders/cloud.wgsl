struct Locals {
    time: f32,
    input: f32}

@group(0) @binding(0) 
var<uniform> locals: Locals;
@group(0) @binding(1)
var resource_texture: texture_2d<f32>;
@group(0) @binding(2)
var effect_sampler: sampler;

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
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(model.position.xy, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = locals.time;
    let intensity = locals.input * 10.0;
    var uv = in.uv;

    var colour = textureSample(resource_texture, effect_sampler, uv);
    let value = max(colour.r, max(colour.g, colour.b));
    let is_cloud = value > 0.2;

    let cloud_colour = vec4<f32>(uv.x * 0.8, uv.y * 0.5, 0.8, 1.0);

    let result = select(colour, cloud_colour, is_cloud);

    return vec4(result.rgb, 0.6);
}

