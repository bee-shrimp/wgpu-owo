// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::config::MAX_ENTITIES;

use crate::ecs::{
    ComponentStorage, Components, CreateEntitySystem, EntityManager, InstanceDataBuildSystem, Pos,
    ToggleSpriteSystem,
};
use crate::renderer::InstanceData;

// ------------------------------------------------------------------- world struct

pub struct World {
    entity_manager: EntityManager,
    pub components: Components,
    instances: Vec<InstanceData>,
    is_running: bool,
}

// ------------------------------------------------------------------- default empty world

impl Default for World {
    /// create empty world.
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            components: Components {
                positions: ComponentStorage::new(),
                sizes: ComponentStorage::new(),
                sprites: ComponentStorage::new(),
            },
            instances: Vec::new(),
            is_running: true,
        }
    }
}

impl World {
    /// initialise world with entities.
    pub fn init(&mut self) -> anyhow::Result<()> {
        CreateEntitySystem::create_entities(&mut self.entity_manager, &mut self.components)
            .context("failed to create entities")?;

        self.instances.reserve(MAX_ENTITIES);

        self.update_instances()?;
        Ok(())
    }

    /// update components.
    pub fn update(&mut self, click_pos: Pos, _dt: f32) -> anyhow::Result<()> {
        ToggleSpriteSystem::update(&mut self.components, click_pos)?;

        self.update_instances()?;

        Ok(())
    }

    /// update instance data.
    pub fn update_instances(&mut self) -> anyhow::Result<()> {
        self.instances.clear();

        let new_instances = InstanceDataBuildSystem.update(&self.components)?;
        self.instances.extend(new_instances);

        Ok(())
    }

    /// pause/unpause.
    pub const fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
    }

    /// return list of InstanceData.
    pub fn get_instances(&self) -> &[InstanceData] {
        &self.instances
    }

    /// return if paused.
    pub const fn is_running(&self) -> bool {
        self.is_running
    }
}
