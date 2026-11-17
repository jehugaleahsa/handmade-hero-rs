use crate::{entity::Entity, high_entity::HighEntity};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Player<'a> {
    entity: &'a Entity,
    high_entity_index: usize,
    high_entity: &'a HighEntity,
    controller_index: usize,
    primary: bool,
    enabled: bool,
}

impl<'a> Player<'a> {
    #[inline]
    #[must_use]
    pub fn new(
        entity: &'a Entity,
        high_entity_index: usize,
        high_entity: &'a HighEntity,
        controller_index: usize,
        primary: bool,
        enabled: bool,
    ) -> Self {
        Self {
            entity,
            high_entity_index,
            high_entity,
            controller_index,
            primary,
            enabled,
        }
    }

    #[inline]
    #[must_use]
    pub fn entity(&self) -> &Entity {
        self.entity
    }

    #[inline]
    #[must_use]
    pub fn high_entity(&self) -> &HighEntity {
        self.high_entity
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PlayerMut<'a> {
    entity: &'a mut Entity,
    high_entity_index: usize,
    high_entity: &'a mut HighEntity,
    controller_index: usize,
    primary: bool,
    enabled: bool,
}

impl<'a> PlayerMut<'a> {
    #[inline]
    #[must_use]
    pub fn new(
        entity: &'a mut Entity,
        high_entity_index: usize,
        high_entity: &'a mut HighEntity,
        controller_index: usize,
        primary: bool,
        enabled: bool,
    ) -> Self {
        Self {
            entity,
            high_entity_index,
            high_entity,
            controller_index,
            primary,
            enabled,
        }
    }

    #[inline]
    #[must_use]
    pub fn entity(&mut self) -> &mut Entity {
        self.entity
    }

    #[inline]
    #[must_use]
    pub fn high_entity(&mut self) -> &mut HighEntity {
        self.high_entity
    }
}
