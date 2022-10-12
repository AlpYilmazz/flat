
// -- Vertex -----

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct Model {
    mx: mat4x4<f32>,
}

struct VertexInput {
    @location(0)    position: vec3<f32>,
    @location(1)    tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position)  clip_position: vec4<f32>,
    @location(0)        tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var<uniform> model: Model;

@vertex
fn vs_main(
    mesh: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * model.mx * vec4<f32>(mesh.position, 1.0);
    out.tex_coords = mesh.tex_coords;
    return out;
}

// -- Fragment -----

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.tex_coords, 0.0, 0.7);
}