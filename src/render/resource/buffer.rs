use bytemuck::{Pod, Zeroable};
use cgmath::{Quaternion, Vector3};
use repr_trait::C;

pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Indices {
    pub fn len(&self) -> usize {
        match self {
            Indices::U16(vec) => vec.len(),
            Indices::U32(vec) => vec.len(),
        }
    }
}

impl Indices {
    pub fn shift(&mut self, offset: u32) {
        match self {
            Indices::U16(vec) => {
                for ind in vec {
                    *ind += offset as u16;
                }
            }
            Indices::U32(vec) => {
                for ind in vec {
                    *ind += offset;
                }
            }
        }
    }

    pub fn extend(&mut self, other: Indices) {
        match (self, other) {
            (Indices::U16(vs), Indices::U16(vo)) => {
                vs.extend(vo);
            }
            (Indices::U32(vs), Indices::U32(vo)) => {
                vs.extend(vo);
            }
            (Indices::U16(vs), Indices::U32(vo)) => {
                vs.extend(vo.iter().map(|a| *a as u16));
            }
            (Indices::U32(vs), Indices::U16(vo)) => {
                vs.extend(vo.iter().map(|a| *a as u32));
            }
        }
    }
}

impl Into<wgpu::IndexFormat> for &Indices {
    fn into(self) -> wgpu::IndexFormat {
        match self {
            Indices::U16(_) => wgpu::IndexFormat::Uint16,
            Indices::U32(_) => wgpu::IndexFormat::Uint32,
        }
    }
}

impl From<Vec<u16>> for Indices {
    fn from(val: Vec<u16>) -> Self {
        Self::U16(val)
    }
}

impl From<Vec<u32>> for Indices {
    fn from(val: Vec<u32>) -> Self {
        Self::U32(val)
    }
}

pub trait MeshVertex: Sized + C + Pod + Zeroable {
    const ATTR_NAMES: &'static [&'static str];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    fn size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }

    // NOTE: I believe my Vertex/Mesh architecture allows 'static Layout
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        }
    }
}

pub trait FromRawVertices: MeshVertex {
    fn from_raw(
        positions: &[f32],
        texcoords: &[f32],
        normals: &[f32],
        vertex_color: &[f32],
    ) -> Vec<Self>;
}

pub trait FromRawVertex: MeshVertex {
    fn from_raw(
        position: &[f32; 3],
        texcoord: &[f32; 2],
        normal: &[f32; 3],
        vertex_color: &[f32; 3],
    ) -> Self;
}

pub trait InstanceUnit: Sized + C + Pod + Zeroable {
    // const ATTR_NAMES: &'static [&'static str];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    fn size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, C, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl MeshVertex for Vertex {
    const ATTR_NAMES: &'static [&'static str] = &["Position", "Texture Coordinates"];

    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
    ];
}

impl FromRawVertex for Vertex {
    fn from_raw(
        position: &[f32; 3],
        texcoord: &[f32; 2],
        _normal: &[f32; 3],
        _vertex_color: &[f32; 3],
    ) -> Self {
        Self {
            position: position.clone(),
            tex_coords: texcoord.clone(),
        }
    }
}

impl FromRawVertices for Vertex {
    fn from_raw(
        positions: &[f32],
        texcoords: &[f32],
        _normals: &[f32],
        _vertex_color: &[f32],
    ) -> Vec<Self> {
        (0..positions.len() / 3)
            .into_iter()
            .map(|i| Vertex {
                position: [positions[i], positions[i + 1], positions[i + 2]],
                tex_coords: [
                    *texcoords.get(i).unwrap_or(&0.0),
                    *texcoords.get(i + 1).unwrap_or(&0.0),
                ],
            })
            .collect()
    }
}

pub struct Instance {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, C, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceUnit for InstanceRaw {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
    ];
}
