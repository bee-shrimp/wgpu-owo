// ---------------------------------------------------------------- imports

use anyhow::Context;

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH, MAX_ENTITIES, RECT_HEIGHT, RECT_WIDTH};

use crate::animation::{AnimationId, AnimationRegistry, init_animation_registry};
use crate::ecs::{
    AnimationSystem, ComponentStorage, Components, CreateEntitySystem, EntityManager,
    InstanceDataBuildSystem, KillEntitySystem, Pos,
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
                animations: ComponentStorage::new(),
            },
            instances: Vec::with_capacity(MAX_ENTITIES),
            anim_registry: AnimationRegistry::new(),
            is_running: true,
        }
    }
}

impl World {
    /// initialises world with entities.
    pub fn init(&mut self) -> anyhow::Result<()> {
        init_animation_registry(&mut self.anim_registry)?;
        let center = Pos {
            x: (LOGIC_WIDTH / 2 - RECT_WIDTH / 2) as f32,
            y: (LOGIC_HEIGHT / 2 - RECT_HEIGHT / 2) as f32,
        };

        CreateEntitySystem::create_entity_with_pos(
            &mut self.entity_manager,
            &mut self.components,
            Some(center),
            AnimationId::new(0),
        )
        .context("failed to create entity")?;

        self.update_instances();
        Ok(())
    }

    /// updates components.
    pub fn update(
        &mut self,
        left_click_pos: Option<Pos>,
        right_click_pos: Option<Pos>,
        dt: f32,
    ) -> anyhow::Result<()> {
        AnimationSystem::update(&mut self.components, &self.anim_registry, dt)?;

        CreateEntitySystem::create_entity_with_pos(
            &mut self.entity_manager,
            &mut self.components,
            left_click_pos,
            AnimationId::new(0),
        )
        .context("failed to create entity")?;

        KillEntitySystem::kill_entity_with_pos(
            &mut self.entity_manager,
            &mut self.components,
            right_click_pos,
        )?;

        self.update_instances();

        println!("{:?}", self.components.positions.count_alive());
        println!("{:?}", self.instances);
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
