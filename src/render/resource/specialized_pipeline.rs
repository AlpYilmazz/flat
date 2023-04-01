use bevy::{prelude::Resource, utils::HashMap};
use std::hash::Hash;

use super::{
    pipeline::{RenderPipelineDescriptor, RenderPipelineId},
    renderer::RenderDevice,
};

pub trait PipelineSpecialize {
    type Key: Hash + Eq;

    fn specialize(&self, render_device: &RenderDevice, key: Self::Key) -> RenderPipelineDescriptor;
}

#[derive(Resource)]
pub struct Specialized<P: PipelineSpecialize> {
    pub pipelines: HashMap<P::Key, RenderPipelineId>,
}

impl<P: PipelineSpecialize> Default for Specialized<P> {
    fn default() -> Self {
        Self {
            pipelines: Default::default(),
        }
    }
}
