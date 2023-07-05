use crate::render::{mesh::Mesh, resource::buffer::VertexBase};

pub const UNIT_TRIANGLE_POSITIONS: &'static [[f32; 3]; 3] = &[
    [0.0, 1.0, 0.0],
    [-0.5, 0.0, 0.0],
    [0.5, 0.0, 0.0]
];

/// Isosceles triangle
/// with unit length base and height
///
pub fn create_unit_triangle() -> Mesh<VertexBase> {
    Mesh::new_with(
        wgpu::PrimitiveTopology::TriangleList,
        UNIT_TRIANGLE_POSITIONS
            .map(|position| VertexBase { position })
            .to_vec(),
        None,
    )
}
