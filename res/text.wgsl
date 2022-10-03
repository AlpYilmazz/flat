
// -- Vertex -----

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0)    position: vec3<f32>,
    @location(1)    tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position)  clip_position: vec4<f32>,
    @location(0)        tex_coords: vec2<f32>,
    @location(1)        color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> model: mat4x4<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = 
        camera.view_proj
        * model
        * vec4<f32>(vertex.position, 1.0);
    out.tex_coords = vertex.tex_coords;

    return out;
}

// -- Frangment -----

@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

fn emult(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.x * b.x, a.y * b.y, a.z * b.z);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse,
        in.tex_coords);

    return vec4<f32>(emult(tex_color.xyz, color.xyz), tex_color.w);
}

