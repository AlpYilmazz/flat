use bevy_reflect::TypeUuid;

use crate::render::{resource::buffer::{Indices, Vertex}, system::RenderAsset};

use super::{Mesh, GpuMesh};

#[derive(TypeUuid)]
#[uuid = "D952EB9F-7AD2-4B1B-B3CE-386735205990"]
pub struct Quad {
    inner_mesh: Mesh<Vertex>,
}

impl Quad {
    pub fn new() -> Self {
        Self {
            inner_mesh: Mesh::new_with(
                wgpu::PrimitiveTopology::TriangleList,
                vec![
                    Vertex {
                        position: [-0.5, 0.5, 0.0],
                        tex_coords: [0.0, 0.0],
                    }, // 0, Upper-Left
                    Vertex {
                        position: [-0.5, -0.5, 0.0],
                        tex_coords: [0.0, 1.0],
                    }, // 1, Lower-Left
                    Vertex {
                        position: [0.5, -0.5, 0.0],
                        tex_coords: [1.0, 1.0],
                    }, // 2, Lower-Right
                    Vertex {
                        position: [0.5, 0.5, 0.0],
                        tex_coords: [1.0, 0.0],
                    }, // 3, Upper-Right
                ],
                Some(Indices::U16(vec![0, 1, 2, 2, 3, 0])),
            ),
        }
    }
}

impl AsRef<Mesh<Vertex>> for Quad {
    fn as_ref(&self) -> &Mesh<Vertex> {
        &self.inner_mesh
    }
}

impl RenderAsset for Quad {
    type ExtractedAsset = GpuMesh;

    fn extract(&self, device: &wgpu::Device, _queue: &wgpu::Queue) -> Self::ExtractedAsset {
        GpuMesh::from_mesh(device, self)
    }
}