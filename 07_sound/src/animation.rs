// ---------------------------------------------------------------- imports

use anyhow::{Context, Result};

use crate::sprite::GridPos;

const MAX_TOTAL_FRAMES: usize = 256;
const MAX_ANIMATIONS: usize = 32;

pub fn init_animation_registry(registry: &mut AnimationRegistry) -> Result<()> {
    const WALK_FRAMES: &[GridPos] = &[
        GridPos { gx: 0, gy: 0 },
        GridPos { gx: 1, gy: 0 },
        GridPos { gx: 2, gy: 0 },
        GridPos { gx: 3, gy: 0 },
        GridPos { gx: 4, gy: 0 },
        GridPos { gx: 5, gy: 0 },
        GridPos { gx: 6, gy: 0 },
        GridPos { gx: 7, gy: 0 },
    ];

    const ENEMY_FRAMES: &[GridPos] = &[
        GridPos { gx: 0, gy: 1 },
        GridPos { gx: 1, gy: 1 },
        GridPos { gx: 2, gy: 1 },
        GridPos { gx: 3, gy: 1 },
        GridPos { gx: 4, gy: 1 },
        GridPos { gx: 5, gy: 1 },
    ];

    registry.register(WALK_FRAMES, 0.16)?;
    registry.register(ENEMY_FRAMES, 0.16)?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct AnimationId {
    pub index: usize,
}
impl AnimationId {
    pub fn new(id: u8) -> Self {
        Self {
            index: usize::from(id),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnimationState {
    pub current_frame: u8,
    pub elapsed: f32,
    pub id: u8,
}

/// static data of frames.
#[derive(Debug, Clone, Copy)]
struct FrameStorage {
    /// flat array of all frames.
    frames: [GridPos; MAX_TOTAL_FRAMES],
    /// number of already registered frames.
    count: usize,
}

impl FrameStorage {
    pub const fn new() -> Self {
        Self {
            frames: [GridPos { gx: 0, gy: 0 }; MAX_TOTAL_FRAMES],
            count: 0,
        }
    }
    /// returns index and size of the animation in frame srorage array.
    pub const fn allocate(&mut self, frame_count: usize) -> Option<(usize, usize)> {
        if self.count + frame_count > MAX_TOTAL_FRAMES {
            return None;
        }
        let offset = self.count;
        self.count += frame_count;
        Some((offset, frame_count))
    }

    /// returns `GridPos` of nth frame of the animation (`GridPos` stored at offset + frame).
    pub fn get(&self, offset: usize, frame: usize) -> GridPos {
        self.frames[offset + frame]
    }
}

/// static definition of animations.
#[derive(Debug, Clone, Copy)]
pub struct AnimationDef {
    /// address in `FrameStorage.frames`.
    pub frame_offset: usize,
    /// how many frames the animation has.
    pub frame_count: usize,
    /// frame changes after this amount of time.
    pub duration_per_frame: f32,
}

/// static data of animations.
pub struct AnimationRegistry {
    /// array of static definitions of animations.
    definitions: [AnimationDef; MAX_ANIMATIONS],
    /// number of already registered animation definitions.
    anim_count: usize,
    /// frame data storage.
    frame_storage: FrameStorage,
}

impl AnimationRegistry {
    pub fn new() -> Self {
        Self {
            definitions: [AnimationDef {
                frame_offset: 0,
                frame_count: 0,
                duration_per_frame: 0.0,
            }; MAX_ANIMATIONS],
            anim_count: 0,
            frame_storage: FrameStorage::new(),
        }
    }

    pub fn register(&mut self, frames: &[GridPos], duration: f32) -> Result<AnimationId> {
        let (offset, count) = self
            .frame_storage
            .allocate(frames.len())
            .context("failed to allocate in frame storage")?;

        self.frame_storage.frames[offset..(frames.len() + offset)].copy_from_slice(frames);

        let anim_id = AnimationId {
            index: self.anim_count,
        };

        self.definitions[self.anim_count] = AnimationDef {
            frame_offset: offset,
            frame_count: count,
            duration_per_frame: duration,
        };
        self.anim_count += 1;
        Ok(anim_id)
    }
    pub fn get_def(&self, anim_id: AnimationId) -> &AnimationDef {
        &self.definitions[anim_id.index]
    }

    pub fn get_frame(&self, anim_id: AnimationId, current_frame: u8) -> GridPos {
        let def = self.get_def(anim_id);
        let idx = usize::from(current_frame) % def.frame_count;
        self.frame_storage.get(def.frame_offset, idx)
    }
}
