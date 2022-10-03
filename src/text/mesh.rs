use crate::render::{mesh::Mesh, resource::buffer::Vertex};

use super::TextAtlas;

pub fn create_screen_text_mesh(atlas: &TextAtlas, src: &str, coord: (f32, f32)) -> Mesh<Vertex> {
    let mut vertices = Vec::with_capacity(src.chars().count());

    let (h, w) = (atlas.h as u32, atlas.w as u32);
    let (mut x, y) = coord;
    for ch in src.chars() {
        let desc = &atlas.descriptors[ch as usize];
        let (tl, br) = atlas.rects[ch as usize].normalized(h, w);

        let decsend = desc.h - desc.bearing_y;
        let x_start = x + desc.bearing_x as f32;
        let y_start = y - decsend as f32;
        let (h, w) = (desc.h as f32, desc.w as f32);

        vertices.extend(&[
            Vertex {
                position: [x_start, y_start + h, 0.0],
                tex_coords: [tl.0, tl.1],
            }, // tl
            Vertex {
                position: [x_start, y_start, 0.0],
                tex_coords: [tl.0, br.1],
            }, // bl
            Vertex {
                position: [x_start + w, y_start, 0.0],
                tex_coords: [br.0, br.1],
            }, // br
            Vertex {
                position: [x_start + w, y_start, 0.0],
                tex_coords: [br.0, br.1],
            }, // br
            Vertex {
                position: [x_start + w, y_start + h, 0.0],
                tex_coords: [br.0, tl.1],
            }, // tr
            Vertex {
                position: [x_start, y_start + h, 0.0],
                tex_coords: [tl.0, tl.1],
            }, // tl
        ]);

        x += (desc.advance >> 6) as f32;
    }

    Mesh::with_all(wgpu::PrimitiveTopology::TriangleList, vertices, None)
}
