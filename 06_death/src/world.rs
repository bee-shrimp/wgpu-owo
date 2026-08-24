// ---------------------------------------------------------------- imports

use anyhow::Context;

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH, MAX_ENTITIES, RECT_HEIGHT, RECT_WIDTH};

use crate::animation::{AnimationId, AnimationRegistry, init_animation_registry};
use crate::ecs::{
    AnimationSystem, ComponentStorage, Components, CreateEntitySystem, Entity, EntityManager,
    InstanceDataBuildSystem, KillEntitySystem, Pos,
};
use crate::renderer::InstanceData;

use crate::app::input::InputState;

// ---------------------------------------------------------------- world struct

pub struct World {
    entity_manager: EntityManager,
    pub components: Components,
    update_targets: Vec<Entity>,
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
            update_targets: Vec::with_capacity(MAX_ENTITIES),
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
            x: f32::from(LOGIC_WIDTH / 2 - RECT_WIDTH / 2),
            y: f32::from(LOGIC_HEIGHT / 2 - RECT_HEIGHT / 2),
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
    pub fn update(&mut self, input_state: InputState, dt: f32) -> anyhow::Result<()> {
        self.update_targets.truncate(0);
        self.update_targets
            .extend(self.components.with_animation_state());

        AnimationSystem::update(
            &mut self.components,
            &self.anim_registry,
            &self.update_targets,
            dt,
        )?;

        CreateEntitySystem::create_entity_with_pos(
            &mut self.entity_manager,
            &mut self.components,
            input_state.left_click,
            AnimationId::new(0),
        )
        .context("failed to create entity")?;

        self.update_targets.truncate(0);
        self.update_targets
            .extend(self.components.with_pos_and_size());

        KillEntitySystem::kill_entity_with_pos(
            &mut self.entity_manager,
            &mut self.components,
            &self.update_targets,
            input_state.right_click,
        )?;

        // self.update_instances();

        Ok(())
    }

    /// updates instance data.
    pub fn update_instances(&mut self) -> &[InstanceData] {
        self.instances.truncate(0);

        self.update_targets.truncate(0);
        self.update_targets.extend(self.components.iter_alive());

        self.instances.extend(InstanceDataBuildSystem::update(
            &self.components,
            &self.anim_registry,
            &self.update_targets,
        ));

        &self.instances
    }

    /// pause/unpause.
    pub const fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
    }

    // /// return list of `InstanceData`.
    // pub fn get_instances(&self) -> &[InstanceData] {
    //     &self.instances
    // }

    /// return if paused.
    pub const fn is_running(&self) -> bool {
        self.is_running
    }
}
