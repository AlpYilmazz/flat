use cgmath::Vector3;

use crate::render::resource::buffer::{Indices, Vertex};

use super::Mesh;

pub fn create_unit_cube() -> Mesh<Vertex> {
    // z grows towards, out of the screen
    // +z .. |screen| .. -z
    // NOTE: CORRECT!!!
    const VERTICES_Z_TOWARDS: &[Vertex] = &[
        Vertex {
            position: [-0.5, -0.5, 0.5],
            tex_coords: [0.0, 1.0],
        }, // 0
        Vertex {
            position: [-0.5, -0.5, -0.5],
            tex_coords: [0.0, 0.0],
        }, // 3
        Vertex {
            position: [0.5, -0.5, -0.5],
            tex_coords: [1.0, 0.0],
        }, // 2
        Vertex {
            position: [0.5, -0.5, 0.5],
            tex_coords: [1.0, 1.0],
        }, // 1
        Vertex {
            position: [-0.5, 0.5, 0.5],
            tex_coords: [0.0, 0.0],
        }, // 4
        Vertex {
            position: [-0.5, -0.5, 0.5],
            tex_coords: [0.0, 1.0],
        }, // 0
        Vertex {
            position: [0.5, -0.5, 0.5],
            tex_coords: [1.0, 1.0],
        }, // 1
        Vertex {
            position: [0.5, 0.5, 0.5],
            tex_coords: [1.0, 0.0],
        }, // 5
        Vertex {
            position: [0.5, 0.5, 0.5],
            tex_coords: [0.0, 0.0],
        }, // 5
        Vertex {
            position: [0.5, -0.5, 0.5],
            tex_coords: [0.0, 1.0],
        }, // 1
        Vertex {
            position: [0.5, -0.5, -0.5],
            tex_coords: [1.0, 1.0],
        }, // 2
        Vertex {
            position: [0.5, 0.5, -0.5],
            tex_coords: [1.0, 0.0],
        }, // 6
        Vertex {
            position: [0.5, 0.5, -0.5],
            tex_coords: [0.0, 0.0],
        }, // 6
        Vertex {
            position: [0.5, -0.5, -0.5],
            tex_coords: [0.0, 1.0],
        }, // 2
        Vertex {
            position: [-0.5, -0.5, -0.5],
            tex_coords: [1.0, 1.0],
        }, // 3
        Vertex {
            position: [-0.5, 0.5, -0.5],
            tex_coords: [1.0, 0.0],
        }, // 7
        Vertex {
            position: [-0.5, 0.5, -0.5],
            tex_coords: [0.0, 0.0],
        }, // 7
        Vertex {
            position: [-0.5, -0.5, -0.5],
            tex_coords: [0.0, 1.0],
        }, // 3
        Vertex {
            position: [-0.5, -0.5, 0.5],
            tex_coords: [1.0, 1.0],
        }, // 0
        Vertex {
            position: [-0.5, 0.5, 0.5],
            tex_coords: [1.0, 0.0],
        }, // 4
        Vertex {
            position: [-0.5, 0.5, -0.5],
            tex_coords: [0.0, 0.0],
        }, // 7
        Vertex {
            position: [-0.5, 0.5, 0.5],
            tex_coords: [0.0, 1.0],
        }, // 4
        Vertex {
            position: [0.5, 0.5, 0.5],
            tex_coords: [1.0, 1.0],
        }, // 5
        Vertex {
            position: [0.5, 0.5, -0.5],
            tex_coords: [1.0, 0.0],
        }, // 6
    ];

    let mut indices = vec![0; 36];
    for i in 0..6 {
        let range = 6 * i..6 * (i + 1);
        indices[range.clone()].copy_from_slice(&[0, 1, 2, 2, 3, 0]);
        for u in &mut indices[range] {
            *u += 4 * i as u16;
        }
    }

    Mesh::with_all(
        wgpu::PrimitiveTopology::TriangleList,
        VERTICES_Z_TOWARDS.to_owned(),
        Some(Indices::U16(indices)),
    )
}

pub enum PlaneAlign {
    XY,
    XZ,
    YZ,
}

impl PlaneAlign {
    pub fn pvector(&self, f: f32, s: f32) -> Vector3<f32> {
        match self {
            PlaneAlign::XY => Vector3::new(f, s, 0.0),   // Normal +Z
            PlaneAlign::XZ => Vector3::new(f, 0.0, -s),  // Normal +Y
            PlaneAlign::YZ => Vector3::new(0.0, -f, -s), // Normal +X
        }
    }
}

pub fn create_aa_plane(
    align: PlaneAlign,
    h: f32,    // fst
    w: f32,    // snd
    rows: u32, // h
    cols: u32, // w
    center: Vector3<f32>,
) -> Mesh<Vertex> {
    let mut vertices = Vec::with_capacity(((rows + 1) * (cols + 1)) as usize);
    let mut indices = Vec::with_capacity((rows * cols * 2 * 3) as usize);

    let lu_corner = center + align.pvector(-w / 2.0, h / 2.0);
    let h_step = h / rows as f32;
    let w_step = w / cols as f32;
    for i in 0..rows + 1 {
        for j in 0..cols + 1 {
            let base = lu_corner + align.pvector(j as f32 * w_step, i as f32 * -h_step);
            vertices.push(Vertex {
                position: [base.x, base.y, base.z],
                tex_coords: [0.5, 0.9],
            });
        }
    }

    let ind = |i: u32, j: u32| -> u32 { i * (cols + 1) + j };
    for i in 0..rows {
        for j in 0..cols {
            indices.extend(&[
                ind(i, j),
                ind(i + 1, j),
                ind(i + 1, j + 1),
                ind(i + 1, j + 1),
                ind(i, j + 1),
                ind(i, j),
            ]);
        }
    }

    Mesh::with_all(
        wgpu::PrimitiveTopology::TriangleList,
        vertices,
        Some(Indices::U32(indices)),
    )
}
