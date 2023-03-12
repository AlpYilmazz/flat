use bevy::utils::HashMap;
use std::hash::Hash;

use super::pipeline::RenderPipelineDescriptor;


pub trait PipelineSpecialize {
    type Key: Hash + Eq;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor;
}

pub struct Specialized<P: PipelineSpecialize> {
    pipelines: HashMap<P::Key, P>,
}