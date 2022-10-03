use noise::{NoiseFn, Perlin, Seedable};

use crate::render::resource::buffer::Vertex;

use super::Mesh;

pub fn randomize_y(mesh: &mut Mesh<Vertex>) {
    let perlin = Perlin::new();
    let perlin = perlin.set_seed(72189);
    // let perlin: Clamp<[f64; 2]> = Clamp::new(&perlin);
    // let perlin = perlin.set_bounds(-10.0, 10.0);
    let vertices_full = mesh.get_vertices_mut();
    let len = vertices_full.len();
    let vertices = &mut vertices_full[0..len / 2];
    for vertex in vertices {
        let coord = [
            0.5 + vertex.position[0] as f64,
            0.5 + vertex.position[2] as f64,
        ];
        let val = perlin.get(coord) as f32;
        vertex.position[1] += val;
    }
}
