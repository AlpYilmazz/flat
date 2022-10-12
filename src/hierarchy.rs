use bevy_ecs::prelude::{Entity, Component};
use smallvec::SmallVec;


#[derive(Debug, Clone, Component, PartialEq, Eq)]
pub struct Parent(pub Entity);
impl Parent {
    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Clone, Component)]
pub struct Children(pub SmallVec<[Entity; 8]>);
impl AsRef<[Entity]> for Children {
    fn as_ref(&self) -> &[Entity] {
        self.0.as_ref()
    }
}
impl AsMut<[Entity]> for Children {
    fn as_mut(&mut self) -> &mut [Entity] {
        self.0.as_mut()
    }
}