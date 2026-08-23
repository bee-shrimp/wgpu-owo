// ---------------------------------------------------------------- imports

use anyhow::Context;

use crate::config::MAX_ENTITIES;

use crate::animation::{AnimationId, AnimationRegistry, init_animation_registry};
use crate::ecs::{
    AnimationSystem, ComponentStorage, Components, CreateEntitySystem, EntityManager,
    InstanceDataBuildSystem, Pos,
};
use crate::renderer::InstanceData;

// ---------------------------------------------------------------- world struct

pub struct World {
    entity_manager: EntityManager,
    pub components: Components,
    instances: Vec<InstanceData>,
    anim_registry: AnimationRegistry,
    is_running: bool,
}

// ---------------------------------------------------------------- default empty world

impl Default for World {
    /// creates empty world.
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            components: Components {
                positions: ComponentStorage::new(),
                sizes: ComponentStorage::new(),
                // sprites: ComponentStorage::new(),
                animations: ComponentStorage::new(),
            },
            instances: Vec::new(),
            anim_registry: AnimationRegistry::new(),
            is_running: true,
        }
    }
}

impl World {
    /// initialises world with entities.
    pub fn init(&mut self) -> anyhow::Result<()> {
        init_animation_registry(&mut self.anim_registry)?;
        CreateEntitySystem::create_grid_entities(
            &mut self.entity_manager,
            &mut self.components,
            // crate::sprite::Sprite::Walk,
            AnimationId::new(1),
        )
        .context("failed to create entities")?;

        self.instances.reserve(MAX_ENTITIES);

        self.update_instances();
        Ok(())
    }

    /// updates components.
    pub fn update(&mut self, _click_pos: Option<Pos>, dt: f32) -> anyhow::Result<()> {
        AnimationSystem::update(&mut self.components, &self.anim_registry, dt)?;
        self.update_instances();

        Ok(())
    }

    /// updates instance data.
    pub fn update_instances(&mut self) {
        self.instances.clear();

        self.instances.extend(InstanceDataBuildSystem::update(
            &self.components,
            &self.anim_registry,
        ));
    }

    /// pause/unpause.
    pub const fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
    }

    /// return list of `InstanceData`.
    pub fn get_instances(&self) -> &[InstanceData] {
        &self.instances
    }

    /// return if paused.
    pub const fn is_running(&self) -> bool {
        self.is_running
    }
}
