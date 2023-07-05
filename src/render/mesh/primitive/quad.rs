use crate::render::{uniform::Color, mesh::Mesh, resource::buffer::{VertexC, Vertex}};

pub const UNIT_SQUARE_POSITIONS: &'static [[f32; 3]; 4] = &[
    [-0.5, 0.5, 0.0],
    [-0.5, -0.5, 0.0],
    [0.5, -0.5, 0.0],
    [0.5, 0.5, 0.0],
];

pub const UNIT_SQUARE_UVS: &'static [[f32; 2]; 4] = &[
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 1.0],
    [1.0, 0.0]
];

pub const UNIT_SQUARE_INDICES: &'static [u16; 6] = &[0, 1, 2, 2, 3, 0];

pub fn create_unit_square_colored() -> Mesh<VertexC> {
    let mut vertices = Vec::new();
    for ind in UNIT_SQUARE_INDICES {
        let position = UNIT_SQUARE_POSITIONS[*ind as usize];
        let uv = UNIT_SQUARE_UVS[*ind as usize];
        let color = Color::WHITE.as_arr();

        vertices.push(VertexC {
            position,
            uv,
            color,
        })
    }

    Mesh::new_with(wgpu::PrimitiveTopology::TriangleList, vertices, None)
}

pub fn create_unit_square() -> Mesh<Vertex> {
    let mut vertices = Vec::new();
    for ind in UNIT_SQUARE_INDICES {
        let position = UNIT_SQUARE_POSITIONS[*ind as usize];
        let uv = UNIT_SQUARE_UVS[*ind as usize];

        vertices.push(Vertex {
            position,
            uv,
        })
    }

    Mesh::new_with(wgpu::PrimitiveTopology::TriangleList, vertices, None)
}
