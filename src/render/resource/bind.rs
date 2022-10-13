use std::num::NonZeroU32;

#[derive(Debug)]
pub struct BindingLayoutEntry {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<NonZeroU32>,
}

impl BindingLayoutEntry {
    pub fn with_binding(self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: self.ty,
            count: self.count,
        }
    }
}

#[derive(Debug)]
pub struct BindingSetLayoutDescriptor {
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindingSetLayoutDescriptor {
    pub fn as_wgpu<'a>(&'a self) -> wgpu::BindGroupLayoutDescriptor<'a> {
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &self.entries,
        }
    }
}

pub trait BindingDesc {
    fn get_layout_entry(&self) -> BindingLayoutEntry;
}

pub trait BindingSetDesc {
    fn layout_desc(&self) -> BindingSetLayoutDescriptor;
    fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout;
}

impl<B0> BindingSetDesc for &B0
where
    B0: BindingDesc,
{
    fn layout_desc(&self) -> BindingSetLayoutDescriptor {
        BindingSetLayoutDescriptor {
            entries: vec![self.get_layout_entry().with_binding(0)],
        }
    }

    fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let bs_layout = self.layout_desc();
        device.create_bind_group_layout(&bs_layout.as_wgpu())
    }
}

pub trait Binding {
    type Desc: BindingDesc;

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a>;
}

pub trait BindingSet {
    type SetDesc: BindingSetDesc;

    fn into_bind_group(&self, device: &wgpu::Device, desc: Self::SetDesc) -> wgpu::BindGroup;
}

#[allow(non_snake_case)]
impl<'B0, B0> BindingSet for &'B0 B0
where
    B0: Binding,
{
    type SetDesc = &'B0 B0::Desc;

    fn into_bind_group(&self, device: &wgpu::Device, desc: Self::SetDesc) -> wgpu::BindGroup {
        let bind_group_layout = desc.bind_group_layout(device);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.get_resource(),
            }],
        });

        bind_group
    }
}

macro_rules! impl_binding_set_tuple {
    ($count:literal,$(($ind: literal,$life: lifetime,$param: ident)),*) => {
        #[allow(non_snake_case)]
        impl<$($life),* , $($param: BindingDesc),*> BindingSetDesc for ($(&$life $param,)*) {
            fn layout_desc(&self) -> BindingSetLayoutDescriptor {
                let ($($param,)*) = *self;
                BindingSetLayoutDescriptor {
                    entries: vec![
                        $(
                            $param.get_layout_entry().with_binding($ind),
                        )*
                    ],
                }
            }
        
            fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
                let bs_layout = self.layout_desc();
                device.create_bind_group_layout(&bs_layout.as_wgpu())
            }
        }

        #[allow(non_snake_case)]
        impl<$($life),* , $($param: Binding),*> BindingSet for ($(&$life $param,)*) {
            type SetDesc = ($(&$life $param::Desc,)*);

            fn into_bind_group(&self, device: &wgpu::Device, desc: Self::SetDesc) -> wgpu::BindGroup {
                let ($($param,)*) = *self;

                let bind_group_layout = desc.bind_group_layout(device);

                let bind_group = device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &bind_group_layout,
                        entries: &[
                            $(
                                wgpu::BindGroupEntry {
                                    binding: $ind,
                                    resource: $param.get_resource(),
                                },
                            )*
                        ],
                    }
                );

                bind_group
            }
        }
    };
}

impl_binding_set_tuple!(1, (0, 'B0, B0));
impl_binding_set_tuple!(2, (0, 'B0, B0), (1, 'B1, B1));
impl_binding_set_tuple!(3, (0, 'B0, B0), (1, 'B1, B1), (2, 'B2, B2));
impl_binding_set_tuple!(4, (0, 'B0, B0), (1, 'B1, B1), (2, 'B2, B2), (3, 'B3, B3));
impl_binding_set_tuple!(5, (0, 'B0, B0), (1, 'B1, B1), (2, 'B2, B2), (3, 'B3, B3), (4, 'B4, B4));
impl_binding_set_tuple!(6, (0, 'B0, B0), (1, 'B1, B1), (2, 'B2, B2), (3, 'B3, B3), (4, 'B4, B4), (5, 'B5, B5));

pub trait AsBindingSet<'a> {
    type Set: BindingSet;

    fn as_binding_set(&'a self) -> Self::Set;
}
pub trait IntoBindingSet {
    type Set: BindingSet;

    fn into_binding_set(self) -> Self::Set;
}
impl<T: BindingSet> IntoBindingSet for T {
    type Set = T;

    fn into_binding_set(self) -> Self::Set {
        self
    }
}

pub trait AsBindingSetDesc<'a> {
    type SetDesc: BindingSetDesc;

    fn as_binding_set(&'a self) -> Self::SetDesc;
}
pub trait IntoBindingSetDesc {
    type SetDesc: BindingSetDesc;

    fn into_binding_set(self) -> Self::SetDesc;
}
impl<T: BindingSetDesc> IntoBindingSetDesc for T {
    type SetDesc = T;

    fn into_binding_set(self) -> Self::SetDesc {
        self
    }
}