
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
    @location(1)    uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position)  clip_position: vec4<f32>,
    @location(0)        uv: vec2<f32>,
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

    return out;
}

// -- Fragment -----

@group(2) @binding(0)
var<uniform> radius: f32;

@group(3) @binding(0)
var<uniform> color: vec4<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (distance(in.uv, vec2<f32>(0.5, 0.5)) <= radius) {
        return color;
    }
    return vec4<f32>(1.0, 0.0, 1.0, 0.1);
}