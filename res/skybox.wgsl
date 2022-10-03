
// -- Vertex -----

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0)    position: vec3<f32>,
    @location(1)    tex_index: i32,
    @location(2)    tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position)  clip_position: vec4<f32>,
    @location(0)        tex_index: i32,
    @location(1)        tex_coords: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
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
    out.tex_index = vertex.tex_index;
    out.tex_coords = vertex.tex_coords;

    return out;
}

// -- Fragment -----

@group(0) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p4 = textureSample(t_diffuse, s_diffuse,
        in.tex_coords, 4);
    let p5 = textureSample(t_diffuse, s_diffuse,
        in.tex_coords, 5);
    let p100 = textureSample(t_diffuse, s_diffuse,
        in.tex_coords, 100);

    return textureSample(t_diffuse, s_diffuse,
        in.tex_coords, in.tex_index);

    // if (in.tex_index == 0) {
    //     return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    // }
    // let layers = textureNumLayers(t_diffuse);
    // if (layers != 6) {
    //     return vec4<f32>(1.0/f32(layers), 0.0, 0.0, 1.0);
    // }

    // if (in.tex_index == 0) {
    //     return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    // }
    // else if (in.tex_index == 1) {
    //     return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    // }
    // else if (in.tex_index == 2) {
    //     return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    // }
    // else if (in.tex_index == 3) {
    //     return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    // }
    // else if (in.tex_index == 4) {
    //     return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    // }
    // else if (in.tex_index == 5) {
    //     return vec4<f32>(0.0, 1.0, 1.0, 1.0);
    // }
    // // return vec4<f32>(1.0/f32(in.tex_index), 0.0, 0.0, 1.0);
    // if (p4.x == p5.x && p4.y == p5.y && p4.z == p5.z && p4.w == p5.w) {
    //     return p100;
    // }
    // return p5;
}