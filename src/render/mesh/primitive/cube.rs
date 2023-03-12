use crate::render::{
    color::Color,
    mesh::Mesh,
    resource::buffer::{Indices, Vertex},
};

use super::FaceDirection;

pub const UNIT_CUBE_POSITIONS: &'static [[f32; 3]; 24] = &[
    // Down, -y, negy
    [-0.5, -0.5, 0.5],  // 0
    [-0.5, -0.5, -0.5], // 3
    [0.5, -0.5, -0.5],  // 2
    [0.5, -0.5, 0.5],   // 1
    // Front, +z, posz
    [-0.5, 0.5, 0.5],  // 4
    [-0.5, -0.5, 0.5], // 0
    [0.5, -0.5, 0.5],  // 1
    [0.5, 0.5, 0.5],   // 5
    // Right, +x, posx
    [0.5, 0.5, 0.5],   // 5
    [0.5, -0.5, 0.5],  // 1
    [0.5, -0.5, -0.5], // 2
    [0.5, 0.5, -0.5],  // 6
    // Back, -z, negz
    [0.5, 0.5, -0.5],   // 6
    [0.5, -0.5, -0.5],  // 2
    [-0.5, -0.5, -0.5], // 3
    [-0.5, 0.5, -0.5],  // 7
    // Left, -x, negx
    [-0.5, 0.5, -0.5],  // 7
    [-0.5, -0.5, -0.5], // 3
    [-0.5, -0.5, 0.5],  // 0
    [-0.5, 0.5, 0.5],   // 4
    // Up, +y, posy
    [-0.5, 0.5, -0.5], // 7
    [-0.5, 0.5, 0.5],  // 4
    [0.5, 0.5, 0.5],   // 5
    [0.5, 0.5, -0.5],  // 6
];

pub const UNIT_CUBE_UVS: &'static [[f32; 2]; 4] = &[
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 1.0],
    [1.0, 0.0]
];

pub const UNIT_CUBE_INDICES_OUTFACE: &'static [u16; 6] = &[0, 1, 2, 2, 3, 0];
pub const UNIT_CUBE_INDICES_INFACE: &'static [u16; 6] = &[0, 2, 1, 2, 0, 3];

pub fn create_unit_cube(facing: FaceDirection) -> Mesh<Vertex> {
    let vertices = UNIT_CUBE_POSITIONS
        .iter()
        .enumerate()
        .map(|(i, vp)| Vertex {
            position: vp.clone(),
            uv: UNIT_CUBE_UVS[i % 4],
            color: Color::WHITE.as_arr(),
        })
        .collect();

    let unit_cube_indices = match facing {
        FaceDirection::In => UNIT_CUBE_INDICES_INFACE,
        FaceDirection::Out => UNIT_CUBE_INDICES_OUTFACE,
    };

    let mut indices: Vec<u16> = vec![0; 36];
    for i in 0..6 {
        let range = 6 * i..6 * (i + 1);
        indices[range.clone()].copy_from_slice(unit_cube_indices);
        for u in &mut indices[range] {
            *u += 4 * i as u16;
        }
    }

    Mesh::new_with(
        wgpu::PrimitiveTopology::TriangleList,
        vertices,
        Some(Indices::U16(indices)),
    )
}
