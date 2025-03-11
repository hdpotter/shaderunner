// ================================
// vertex shader
// ================================
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.position.x, model.position.y, 0.0, 1.0);

    return out;
}

@group(0) @binding(0)
var sprite_texture: texture_2d<f32>;
@group(0) @binding(1)
var sprite_sampler: sampler;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // return vec4<f32>(in.uv.x, 0.0, in.uv.y, 1.0);

    return textureSample(sprite_texture, sprite_sampler, in.uv);
}