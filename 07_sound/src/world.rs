// ---------------------------------------------------------------- imports

use crate::config::MAX_ENTITIES;

use crate::animation::AnimationRegistry;

use crate::ecs::{ComponentStorage, Components, Entity, EntityManager, Systems};
use crate::renderer::InstanceData;

use crate::app::input::InputState;
use crate::sound::SoundPlayer;

pub struct WorldData {
    pub entity_manager: EntityManager,
    pub components: Components,
    pub update_targets: Vec<Entity>,
    pub instances: Vec<InstanceData>,
    pub anim_registry: AnimationRegistry,
}
impl Default for WorldData {
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
        }
    }
}

// ---------------------------------------------------------------- world struct

pub struct World {
    data: WorldData,
    systems: Systems,
    is_running: bool,
}

// ---------------------------------------------------------------- default empty world

impl Default for World {
    /// creates empty world.
    fn default() -> Self {
        Self {
            data: WorldData::default(),
            systems: Systems::new(),
            is_running: true,
        }
    }
}

impl World {
    /// initialises world with entities.
    pub fn init(&mut self) -> anyhow::Result<()> {
        self.systems.init(&mut self.data)?;
        self.update_instances();
        Ok(())
    }

    /// updates components.
    pub fn update(
        &mut self,
        input: &InputState,
        sound: &SoundPlayer,
        dt: f32,
    ) -> anyhow::Result<()> {
        self.systems.update(&mut self.data, &input, sound, dt)?;

        // self.update_instances();

        Ok(())
    }

    /// updates instance data.
    pub fn update_instances(&mut self) -> &[InstanceData] {
        self.systems.update_instances(&mut self.data);

        &self.data.instances
    }

    /// pause/unpause.
    pub const fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
    }

    /// return if paused.
    pub const fn is_running(&self) -> bool {
        self.is_running
    }
}
