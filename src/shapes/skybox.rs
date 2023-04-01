use crate::render::{
    mesh::{
        primitive::{cube::create_unit_cube, FaceDirection},
        Mesh,
    },
    resource::buffer::VertexTex3,
};

pub const SIDES: [&'static str; 6] = [
    // "negy", "posz", "posx",
    // "negz", "negx", "posy",
    "negy", "posz", "posx", "negz", "negx", "posy",
];

const SKYBOX_UVS: &'static [[f32; 3]; 24] = &[
    // Down, -y, negy
    [0.0, 1.0, 0.0], // 0
    [0.0, 0.0, 0.0], // 3
    [1.0, 0.0, 0.0], // 2
    [1.0, 1.0, 0.0], // 1
    // Front, +z, posz
    [1.0, 0.0, 1.0], // 4
    [1.0, 1.0, 1.0], // 0
    [0.0, 1.0, 1.0], // 1
    [0.0, 0.0, 1.0], // 5
    // Right, +x, posx
    [1.0, 0.0, 2.0], // 5
    [1.0, 1.0, 2.0], // 1
    [0.0, 1.0, 2.0], // 2
    [0.0, 0.0, 2.0], // 6
    // Back, -z, negz
    [1.0, 0.0, 3.0], // 6
    [1.0, 1.0, 3.0], // 2
    [0.0, 1.0, 3.0], // 3
    [0.0, 0.0, 3.0], // 7
    // Left, -x, negx
    [1.0, 0.0, 4.0], // 7
    [1.0, 1.0, 4.0], // 3
    [0.0, 1.0, 4.0], // 0
    [0.0, 0.0, 4.0], // 4
    // Up, +y, posy
    [0.0, 1.0, 5.0], // 7
    [0.0, 0.0, 5.0], // 4
    [1.0, 0.0, 5.0], // 5
    [1.0, 1.0, 5.0], // 6
];

pub fn create_skybox() -> Mesh<VertexTex3> {
    let unit_cube = create_unit_cube(FaceDirection::In).consume();

    let skybox_vertices = unit_cube
        .vertices
        .into_iter()
        .enumerate()
        .map(|(i, v)| VertexTex3 {
            position: v.position,
            uv: SKYBOX_UVS[i],
            color: v.color,
        })
        .collect();

    Mesh::new_with(
        unit_cube.primitive_topology,
        skybox_vertices,
        unit_cube.indices,
    )
}
