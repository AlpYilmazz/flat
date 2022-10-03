use wgpu::util::DeviceExt;

use super::resource::buffer::Indices;

pub struct MeshVertexBufferLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

pub struct Mesh {
    primitive_topology: wgpu::PrimitiveTopology,
    attributes: Vec<MeshAttribute>,
    indices: Option<Indices>,
}

impl Mesh {
    pub fn new(primitive_topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            primitive_topology,
            attributes: Default::default(),
            indices: None,
        }
    }

    pub fn get_indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }

    pub fn set_indices(&mut self, indices: Indices) {
        self.indices = Some(indices);
    }

    pub fn add_attribute(
        &mut self,
        descriptor: VertexAttributeDescriptor,
        values: impl Into<VertexAttributeValues>,
    ) {
        let values = values.into();
        let values_format: wgpu::VertexFormat = (&values).into(); // From::from(&values);

        if descriptor.format != values_format {
            panic!("Vertex values does not match the format");
        }

        self.attributes.push(MeshAttribute { descriptor, values });
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

    pub fn get_vertex_buffer_layout(&self) -> MeshVertexBufferLayout {
        let mut offset = 0;
        let attributes: Vec<wgpu::VertexAttribute> = self.attributes[..]
            .into_iter()
            .enumerate()
            .map(|(i, attr)| {
                let va = wgpu::VertexAttribute {
                    format: (&attr.values).into(),
                    offset: offset as wgpu::BufferAddress,
                    shader_location: i as u32,
                };
                offset += attr.descriptor.format.get_size();

                va
            })
            .collect();
        MeshVertexBufferLayout {
            array_stride: self.get_vertex_size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes,
        }
    }

    pub fn get_vertex_buffer_bytes(&self) -> Vec<u8> {
        let vertex_size = self.get_vertex_size() as usize;
        let vertex_count = self.count_vertices();

        let mut vertex_buffer_bytes = vec![0; vertex_size * vertex_count];

        let mut attr_offset = 0;
        for attribute in &self.attributes {
            let attr_size = attribute.descriptor.format.get_size() as usize;
            let attr_bytes: &[u8] = attribute.values.get_bytes();
            for (vertex_index, attr_bytes) in attr_bytes.chunks_exact(attr_size).enumerate() {
                let offset = vertex_index * vertex_size + attr_offset;
                vertex_buffer_bytes[offset..offset + attr_size].copy_from_slice(attr_bytes);
            }
            attr_offset += attr_size;
        }

        vertex_buffer_bytes
    }

    pub fn get_vertex_size(&self) -> u64 {
        let mut vertex_size = 0;
        for attr in &self.attributes {
            vertex_size += attr.descriptor.format.size();
        }
        vertex_size
    }

    // TODO: should I check for vertex counts being same
    pub fn count_vertices(&self) -> usize {
        self.attributes
            .get(0)
            .map(|attr0| attr0.values.len())
            .unwrap_or(0)
    }
}

pub struct MeshAttribute {
    descriptor: VertexAttributeDescriptor,
    values: VertexAttributeValues,
}

pub struct VertexAttributeDescriptor {
    pub name: &'static str,
    pub format: wgpu::VertexFormat,
}

pub trait VertexFormatSize {
    fn get_size(&self) -> u64;
}

impl VertexFormatSize for wgpu::VertexFormat {
    fn get_size(&self) -> u64 {
        match self {
            wgpu::VertexFormat::Uint8x2 => 2,
            wgpu::VertexFormat::Uint8x4 => 4,
            wgpu::VertexFormat::Sint8x2 => 2,
            wgpu::VertexFormat::Sint8x4 => 4,
            wgpu::VertexFormat::Unorm8x2 => 2,
            wgpu::VertexFormat::Unorm8x4 => 4,
            wgpu::VertexFormat::Snorm8x2 => 2,
            wgpu::VertexFormat::Snorm8x4 => 4,
            wgpu::VertexFormat::Uint16x2 => 2 * 2,
            wgpu::VertexFormat::Uint16x4 => 2 * 4,
            wgpu::VertexFormat::Sint16x2 => 2 * 2,
            wgpu::VertexFormat::Sint16x4 => 2 * 4,
            wgpu::VertexFormat::Unorm16x2 => 2 * 2,
            wgpu::VertexFormat::Unorm16x4 => 2 * 4,
            wgpu::VertexFormat::Snorm16x2 => 2 * 2,
            wgpu::VertexFormat::Snorm16x4 => 2 * 4,
            wgpu::VertexFormat::Float16x2 => 2 * 2,
            wgpu::VertexFormat::Float16x4 => 2 * 4,
            wgpu::VertexFormat::Float32 => 4,
            wgpu::VertexFormat::Float32x2 => 4 * 2,
            wgpu::VertexFormat::Float32x3 => 4 * 3,
            wgpu::VertexFormat::Float32x4 => 4 * 4,
            wgpu::VertexFormat::Uint32 => 4,
            wgpu::VertexFormat::Uint32x2 => 4 * 2,
            wgpu::VertexFormat::Uint32x3 => 4 * 3,
            wgpu::VertexFormat::Uint32x4 => 4 * 4,
            wgpu::VertexFormat::Sint32 => 4,
            wgpu::VertexFormat::Sint32x2 => 4 * 2,
            wgpu::VertexFormat::Sint32x3 => 4 * 3,
            wgpu::VertexFormat::Sint32x4 => 4 * 4,
            wgpu::VertexFormat::Float64 => 8,
            wgpu::VertexFormat::Float64x2 => 8 * 2,
            wgpu::VertexFormat::Float64x3 => 8 * 3,
            wgpu::VertexFormat::Float64x4 => 8 * 4,
        }
    }
}

pub enum VertexAttributeValues {
    Float32(Vec<f32>),
    Sint32(Vec<i32>),
    Uint32(Vec<u32>),
    Float32x2(Vec<[f32; 2]>),
    Sint32x2(Vec<[i32; 2]>),
    Uint32x2(Vec<[u32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Sint32x3(Vec<[i32; 3]>),
    Uint32x3(Vec<[u32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Sint32x4(Vec<[i32; 4]>),
    Uint32x4(Vec<[u32; 4]>),
    Sint16x2(Vec<[i16; 2]>),
    Snorm16x2(Vec<[i16; 2]>),
    Uint16x2(Vec<[u16; 2]>),
    Unorm16x2(Vec<[u16; 2]>),
    Sint16x4(Vec<[i16; 4]>),
    Snorm16x4(Vec<[i16; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Unorm16x4(Vec<[u16; 4]>),
    Sint8x2(Vec<[i8; 2]>),
    Snorm8x2(Vec<[i8; 2]>),
    Uint8x2(Vec<[u8; 2]>),
    Unorm8x2(Vec<[u8; 2]>),
    Sint8x4(Vec<[i8; 4]>),
    Snorm8x4(Vec<[i8; 4]>),
    Uint8x4(Vec<[u8; 4]>),
    Unorm8x4(Vec<[u8; 4]>),
}

impl VertexAttributeValues {
    pub fn len(&self) -> usize {
        match *self {
            VertexAttributeValues::Float32(ref values) => values.len(),
            VertexAttributeValues::Sint32(ref values) => values.len(),
            VertexAttributeValues::Uint32(ref values) => values.len(),
            VertexAttributeValues::Float32x2(ref values) => values.len(),
            VertexAttributeValues::Sint32x2(ref values) => values.len(),
            VertexAttributeValues::Uint32x2(ref values) => values.len(),
            VertexAttributeValues::Float32x3(ref values) => values.len(),
            VertexAttributeValues::Sint32x3(ref values) => values.len(),
            VertexAttributeValues::Uint32x3(ref values) => values.len(),
            VertexAttributeValues::Float32x4(ref values) => values.len(),
            VertexAttributeValues::Sint32x4(ref values) => values.len(),
            VertexAttributeValues::Uint32x4(ref values) => values.len(),
            VertexAttributeValues::Sint16x2(ref values) => values.len(),
            VertexAttributeValues::Snorm16x2(ref values) => values.len(),
            VertexAttributeValues::Uint16x2(ref values) => values.len(),
            VertexAttributeValues::Unorm16x2(ref values) => values.len(),
            VertexAttributeValues::Sint16x4(ref values) => values.len(),
            VertexAttributeValues::Snorm16x4(ref values) => values.len(),
            VertexAttributeValues::Uint16x4(ref values) => values.len(),
            VertexAttributeValues::Unorm16x4(ref values) => values.len(),
            VertexAttributeValues::Sint8x2(ref values) => values.len(),
            VertexAttributeValues::Snorm8x2(ref values) => values.len(),
            VertexAttributeValues::Uint8x2(ref values) => values.len(),
            VertexAttributeValues::Unorm8x2(ref values) => values.len(),
            VertexAttributeValues::Sint8x4(ref values) => values.len(),
            VertexAttributeValues::Snorm8x4(ref values) => values.len(),
            VertexAttributeValues::Uint8x4(ref values) => values.len(),
            VertexAttributeValues::Unorm8x4(ref values) => values.len(),
        }
    }

    pub fn get_bytes(&self) -> &[u8] {
        use bytemuck::cast_slice;
        match self {
            VertexAttributeValues::Float32(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint32(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint32(values) => cast_slice(&values[..]),
            VertexAttributeValues::Float32x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint32x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint32x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Float32x3(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint32x3(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint32x3(values) => cast_slice(&values[..]),
            VertexAttributeValues::Float32x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint32x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint32x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint16x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Snorm16x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint16x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Unorm16x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint16x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Snorm16x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint16x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Unorm16x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint8x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Snorm8x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint8x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Unorm8x2(values) => cast_slice(&values[..]),
            VertexAttributeValues::Sint8x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Snorm8x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Uint8x4(values) => cast_slice(&values[..]),
            VertexAttributeValues::Unorm8x4(values) => cast_slice(&values[..]),
        }
    }
}

impl From<&VertexAttributeValues> for wgpu::VertexFormat {
    fn from(values: &VertexAttributeValues) -> Self {
        match values {
            VertexAttributeValues::Float32(_) => wgpu::VertexFormat::Float32,
            VertexAttributeValues::Sint32(_) => wgpu::VertexFormat::Sint32,
            VertexAttributeValues::Uint32(_) => wgpu::VertexFormat::Uint32,
            VertexAttributeValues::Float32x2(_) => wgpu::VertexFormat::Float32x2,
            VertexAttributeValues::Sint32x2(_) => wgpu::VertexFormat::Sint32x2,
            VertexAttributeValues::Uint32x2(_) => wgpu::VertexFormat::Uint32x2,
            VertexAttributeValues::Float32x3(_) => wgpu::VertexFormat::Float32x3,
            VertexAttributeValues::Sint32x3(_) => wgpu::VertexFormat::Sint32x3,
            VertexAttributeValues::Uint32x3(_) => wgpu::VertexFormat::Uint32x3,
            VertexAttributeValues::Float32x4(_) => wgpu::VertexFormat::Float32x4,
            VertexAttributeValues::Sint32x4(_) => wgpu::VertexFormat::Sint32x4,
            VertexAttributeValues::Uint32x4(_) => wgpu::VertexFormat::Uint32x4,
            VertexAttributeValues::Sint16x2(_) => wgpu::VertexFormat::Sint16x2,
            VertexAttributeValues::Snorm16x2(_) => wgpu::VertexFormat::Snorm16x2,
            VertexAttributeValues::Uint16x2(_) => wgpu::VertexFormat::Uint16x2,
            VertexAttributeValues::Unorm16x2(_) => wgpu::VertexFormat::Unorm16x2,
            VertexAttributeValues::Sint16x4(_) => wgpu::VertexFormat::Sint16x4,
            VertexAttributeValues::Snorm16x4(_) => wgpu::VertexFormat::Snorm16x4,
            VertexAttributeValues::Uint16x4(_) => wgpu::VertexFormat::Uint16x4,
            VertexAttributeValues::Unorm16x4(_) => wgpu::VertexFormat::Unorm16x4,
            VertexAttributeValues::Sint8x2(_) => wgpu::VertexFormat::Sint8x2,
            VertexAttributeValues::Snorm8x2(_) => wgpu::VertexFormat::Snorm8x2,
            VertexAttributeValues::Uint8x2(_) => wgpu::VertexFormat::Uint8x2,
            VertexAttributeValues::Unorm8x2(_) => wgpu::VertexFormat::Unorm8x2,
            VertexAttributeValues::Sint8x4(_) => wgpu::VertexFormat::Sint8x4,
            VertexAttributeValues::Snorm8x4(_) => wgpu::VertexFormat::Snorm8x4,
            VertexAttributeValues::Uint8x4(_) => wgpu::VertexFormat::Uint8x4,
            VertexAttributeValues::Unorm8x4(_) => wgpu::VertexFormat::Unorm8x4,
        }
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

pub struct GpuMesh {
    pub mesh_vertex_buffer_layout: MeshVertexBufferLayout,
    pub vertex_buffer: wgpu::Buffer,
    pub assembly: GpuMeshAssembly,
    pub primitive_topology: wgpu::PrimitiveTopology,
}

impl GpuMesh {
    pub fn from_mesh(mesh: &Mesh, device: &wgpu::Device) -> GpuMesh {
        GpuMesh {
            mesh_vertex_buffer_layout: mesh.get_vertex_buffer_layout(),
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
                    vertex_count: mesh.count_vertices(),
                },
            },
            primitive_topology: mesh.get_primitive_topology(),
        }
    }
}
