use bevy_ecs::prelude::Component;
use bevy_reflect::TypeUuid;
use wgpu::util::DeviceExt;

use super::{
    resource::buffer::{FromRawVertex, Indices, MeshVertex},
    system::{RenderAsset, Draw},
};

pub mod extend;
pub mod primitive;
pub mod util;

pub struct Model<V: MeshVertex> {
    pub meshes: Vec<Mesh<V>>,
}

#[derive(TypeUuid)]
#[uuid = "8628FE7C-A4E9-4056-91BD-FD6AA7817E39"]
pub struct Mesh<V: MeshVertex> {
    primitive_topology: wgpu::PrimitiveTopology,
    vertices: Vec<V>,
    indices: Option<Indices>,
}

impl<V: MeshVertex> Mesh<V> {
    const ZERO: f32 = 0.0;

    pub fn new(primitive_topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            primitive_topology,
            vertices: Default::default(),
            indices: None,
        }
    }

    pub fn new_with(
        primitive_topology: wgpu::PrimitiveTopology,
        vertices: Vec<V>,
        indices: Option<Indices>,
    ) -> Self {
        Self {
            primitive_topology,
            vertices,
            indices,
        }
    }

    pub fn load_obj(filepath: &str) -> Model<V>
    where
        V: FromRawVertex,
    {
        let (models, _) = tobj::load_obj(filepath, &tobj::GPU_LOAD_OPTIONS)
            .expect("Obj file could not be loaded");

        let meshes: Vec<Mesh<V>> = models
            .into_iter()
            .map(|model| {
                let vertices: Vec<V> = (0..model.mesh.positions.len() / 3)
                    .into_iter()
                    .map(|i| {
                        V::from_raw(
                            &model.mesh.positions.as_slice()[i..i + 3]
                                .try_into()
                                .unwrap(),
                            &[
                                *model.mesh.texcoords.get(i).unwrap_or(&Self::ZERO),
                                *model.mesh.texcoords.get(i + 1).unwrap_or(&Self::ZERO),
                            ],
                            &[
                                *model.mesh.normals.get(i).unwrap_or(&Self::ZERO),
                                *model.mesh.normals.get(i + 1).unwrap_or(&Self::ZERO),
                                *model.mesh.normals.get(i + 2).unwrap_or(&Self::ZERO),
                            ],
                            &[
                                *model.mesh.vertex_color.get(i).unwrap_or(&Self::ZERO),
                                *model.mesh.vertex_color.get(i + 1).unwrap_or(&Self::ZERO),
                                *model.mesh.vertex_color.get(i + 2).unwrap_or(&Self::ZERO),
                            ],
                            // &[0.0, 0.0],
                            // &[0.0, 0.0, 0.0],
                            // &[0.0, 0.0, 0.0],
                            // &model.mesh.texcoords.as_slice()[i..i+2].try_into().unwrap_or([0.0, 0.0]),
                            // &model.mesh.normals.as_slice()[i..i+3].try_into().unwrap_or([0.0, 0.0, 0.0]),
                            // &model.mesh.vertex_color.as_slice()[i..i+3].try_into().unwrap_or([0.0, 0.0, 0.0]),
                        )
                    })
                    .collect();

                // V::from_raw(
                //     &model.mesh.positions,
                //     &model.mesh.texcoords,
                //     &model.mesh.normals,
                //     &model.mesh.vertex_color
                // );

                Self::new_with(
                    wgpu::PrimitiveTopology::TriangleList,
                    vertices,
                    Some(Indices::U32(model.mesh.indices)),
                )
            })
            .collect();

        Model { meshes }
    }

    pub fn get_vertices(&self) -> &[V] {
        &self.vertices
    }

    pub fn get_vertices_mut(&mut self) -> &mut [V] {
        &mut self.vertices
    }

    pub fn set_vertices(&mut self, vertices: Vec<V>) {
        self.vertices = vertices;
    }

    pub fn push_vertices(&mut self, vertices: impl IntoIterator<Item = V>) {
        self.vertices.extend(vertices);
    }

    pub fn get_indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }

    pub fn get_indices_mut(&mut self) -> Option<&mut Indices> {
        self.indices.as_mut()
    }

    pub fn set_indices(&mut self, indices: Indices) {
        self.indices = Some(indices);
    }

    pub fn get_primitive_topology(&self) -> wgpu::PrimitiveTopology {
        self.primitive_topology
    }

    pub fn get_index_buffer_bytes(&self) -> Option<&[u8]> {
        self.indices.as_ref().map(|inds| match inds {
            Indices::U16(ivals) => bytemuck::cast_slice(&ivals[..]),
            Indices::U32(ivals) => bytemuck::cast_slice(&ivals[..]),
        })
    }

    pub fn get_vertex_buffer_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    pub fn get_vertex_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static> {
        // TODO: lifetime
        V::layout()
    }

    pub fn vertex_size(&self) -> u64 {
        V::size()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

impl<V: MeshVertex> AsRef<Self> for Mesh<V> {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(TypeUuid)]
#[uuid = "ED280816-E404-444A-A2D9-FFD2D171F928"]
pub struct BatchMesh<V: MeshVertex> {
    indexed: bool,
    inner_mesh: Mesh<V>,
}

impl<V: MeshVertex> BatchMesh<V> {
    pub fn new(primitive_topology: wgpu::PrimitiveTopology, indexed: bool) -> Self {
        Self {
            indexed,
            inner_mesh: Mesh::new(primitive_topology),
        }
    }

    pub fn add(&mut self, mesh: Mesh<V>) {
        let (vertices, indices) = (mesh.vertices, mesh.indices);
        let offset = vertices.len() as u32;

        self.inner_mesh.push_vertices(vertices);

        match self.inner_mesh.get_indices_mut() {
            Some(inner_indices) => {
                match indices {
                    Some(mut indices) => {
                        indices.shift(offset);
                        inner_indices.extend(indices);
                    }
                    // TODO: OR: may convert non-indexed into indexed
                    // by triplet indexing
                    None => panic!("Index requirements does not match"),
                }
            }
            None => {
                match (self.indexed, indices) {
                    (true, Some(mut indices)) => {
                        indices.shift(offset);
                        self.inner_mesh.set_indices(indices);
                    }
                    (false, None) => {
                        // Normal Case
                    }
                    // TODO: OR: may produce garbage gracefully
                    _ => panic!("Index requirements does not match"),
                }
            }
        }
    }

    pub fn add_all(&mut self, meshes: impl IntoIterator<Item = Mesh<V>>) {
        for mesh in meshes {
            self.add(mesh);
        }
    }
}

impl<V: MeshVertex> AsRef<Mesh<V>> for BatchMesh<V> {
    fn as_ref(&self) -> &Mesh<V> {
        &self.inner_mesh
    }
}

pub enum GpuMeshAssembly {
    Indexed {
        index_buffer: wgpu::Buffer,
        index_count: usize,
        index_format: wgpu::IndexFormat,
    },
    NonIndexed {
        vertex_count: usize,
    },
}

#[derive(Component)]
pub struct GpuMesh {
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>, // TODO: lifetime again
    pub vertex_buffer: wgpu::Buffer,
    pub assembly: GpuMeshAssembly,
    pub primitive_topology: wgpu::PrimitiveTopology,
}

impl GpuMesh {
    pub fn from_mesh<V, M>(device: &wgpu::Device, mesh: M) -> GpuMesh
    where
        V: MeshVertex,
        M: AsRef<Mesh<V>>,
    {
        let mesh: &Mesh<V> = mesh.as_ref();
        GpuMesh {
            vertex_buffer_layout: mesh.get_vertex_buffer_layout(),
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &mesh.get_vertex_buffer_bytes(),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            assembly: match mesh.get_index_buffer_bytes() {
                Some(indices) => GpuMeshAssembly::Indexed {
                    index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: indices,
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                    index_count: mesh.get_indices().unwrap().len(),
                    index_format: mesh.get_indices().unwrap().into(),
                },
                None => GpuMeshAssembly::NonIndexed {
                    vertex_count: mesh.vertex_count(),
                },
            },
            primitive_topology: mesh.get_primitive_topology(),
        }
    }
}

impl<V: MeshVertex> RenderAsset for Mesh<V> {
    type ExtractedAsset = GpuMesh;

    fn extract(&self, device: &wgpu::Device, _queue: &wgpu::Queue) -> Self::ExtractedAsset {
        GpuMesh::from_mesh(&device, self)
    }
}

impl<V: MeshVertex> RenderAsset for BatchMesh<V> {
    type ExtractedAsset = GpuMesh;

    fn extract(&self, device: &wgpu::Device, _queue: &wgpu::Queue) -> Self::ExtractedAsset {
        GpuMesh::from_mesh(&device, self)
    }
}

impl Draw for GpuMesh {
    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_data: Option<&'a super::InstanceData>,
    ) {
        let mut instance_count = 1;
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        if let Some(instance_data) = instance_data {
            render_pass.set_vertex_buffer(1, instance_data.0.slice(..));
            instance_count = instance_data.1;
        }

        match &self.assembly {
            GpuMeshAssembly::Indexed {
                index_buffer,
                index_count,
                index_format,
            } => {
                render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
                render_pass.draw_indexed(0..*index_count as u32, 0, 0..instance_count);
            }
            GpuMeshAssembly::NonIndexed { vertex_count } => {
                render_pass.draw(0..*vertex_count as u32, 0..instance_count);
            }
        }
    }
}
