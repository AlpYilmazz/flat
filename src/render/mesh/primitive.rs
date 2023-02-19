use crate::render::{resource::buffer::Vertex, color::Color};

use super::Mesh;


pub const UNIT_SQUARE_CORNERS: &'static [[f32; 3]; 4] =
    &[
        [-0.5, 0.5, 0.0],
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
    ];
pub const UNIT_SQUARE_UVS: &'static [[f32; 2]; 4] =
    &[
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
    ];
pub const UNIT_SQUARE_INDICES: &'static [usize; 6] =
    &[
        0, 1, 2,
        2, 3, 0
    ];

pub fn create_unit_square() -> Mesh<Vertex> {
    let mut vertices = Vec::new();
    for ind in UNIT_SQUARE_INDICES {
        let position = UNIT_SQUARE_CORNERS[*ind];
        let uv = UNIT_SQUARE_UVS[*ind];
        let color = Color::WHITE.as_arr();

        vertices.push(Vertex {
            position,
            uv,
            color,
        })
    }
    
    Mesh::new_with(
        wgpu::PrimitiveTopology::TriangleList,
        vertices,
        None,
    )
}
