use bevy::prelude::*;

/// Marker for all mob entities.
#[derive(Component)]
pub struct Mob;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MobType {
    Zombie,
    Skeleton,
    Pig,
}

/// AI state machine for mobs.
#[derive(Component)]
pub enum MobAiState {
    Idle { timer: f32 },
    Wander { target: Vec3, timer: f32 },
    Chase { target_entity: Entity },
    Attack { cooldown: f32 },
    Flee { direction: Vec3, timer: f32 },
}

impl Default for MobAiState {
    fn default() -> Self {
        Self::Idle { timer: 2.0 }
    }
}

/// Axis-aligned bounding box collider for entity raycast.
#[derive(Component)]
pub struct AabbCollider {
    pub half_extents: Vec3,
}

/// Mob velocity (separate from PlayerController).
#[derive(Component, Default)]
pub struct MobVelocity {
    pub value: Vec3,
}

/// Tracks mob spawn counts and timing.
#[derive(Resource)]
pub struct MobSpawnTracker {
    pub spawn_timer: f32,
    pub max_hostiles: u32,
    pub max_passives: u32,
}

impl Default for MobSpawnTracker {
    fn default() -> Self {
        Self {
            spawn_timer: 0.0,
            max_hostiles: 15,
            max_passives: 10,
        }
    }
}

/// Marker for hostile mobs that despawn at dawn.
#[derive(Component)]
pub struct DespawnAtDay;
