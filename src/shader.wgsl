// ================================
// vertex shader
// ================================
struct InstanceInput {
    @location(5) model_matrix0: vec4<f32>,
    @location(6) model_matrix1: vec4<f32>,
    @location(7) model_matrix2: vec4<f32>,
    @location(8) model_matrix3: vec4<f32>,

    @location(9) normal_matrix0: vec3<f32>,
    @location(10) normal_matrix1: vec3<f32>,
    @location(11) normal_matrix2: vec3<f32>,
};

struct CameraUniform {
    position: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct LightUniform {
    direction: vec3<f32>,
    color: vec3<f32>,
    ambient_color: vec3<f32>,
};
@group(0) @binding(1)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix0,
        instance.model_matrix1,
        instance.model_matrix2,
        instance.model_matrix3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix0,
        instance.normal_matrix1,
        instance.normal_matrix2,
    );

    var out: VertexOutput;
    out.color = model.color;

    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    out.clip_position = camera.view_proj * world_position;

    out.world_normal = normal_matrix * model.normal;

    return out;
}

// ================================
// fragment shader
// ================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_color = light.ambient_color;
    
    let diffuse_strength = max(dot(in.world_normal, -light.direction), 0.0);
    let diffuse_color = diffuse_strength * light.color;

    let result = (ambient_color + diffuse_color) * in.color;

    return vec4<f32>(result, 1.0);

    // return vec4<f32>(in.color, 1.0);
}








