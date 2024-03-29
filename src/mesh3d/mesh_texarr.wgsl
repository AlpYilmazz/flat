
// -- Vertex -----

struct Camera {
    view_proj: mat4x4<f32>,
    // inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    // inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    // inverse_projection: mat4x4<f32>,
    // world_position: vec3<f32>,
    // viewport(x_origin, y_origin, width, height)
    // viewport: vec4<f32>,
}

struct Model {
    model: mat4x4<f32>,
}

struct VertexInput {
    @location(0)    position: vec3<f32>,
    @location(1)    uv: vec3<f32>,
    @location(2)    color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position)  clip_position: vec4<f32>,
    @location(0)        uv: vec3<f32>,
    @location(2)        color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> model: Model;

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * model.model * vec4<f32>(vertex.position, 1.0);
    out.uv = vertex.uv;
    out.color = vertex.color;

    return out;
}

// -- Fragment -----

@group(2) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color = textureSample(t_diffuse, s_diffuse, in.uv.xy, i32(in.uv.z));
    tex_color += in.color;

    return tex_color;
}