pub mod ai;
pub mod components;
pub mod loot;
pub mod projectile;
pub mod spawning;

use bevy::prelude::*;

use crate::states::GameState;
use components::MobSpawnTracker;

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MobSpawnTracker>()
            .add_systems(Update, (
                spawning::mob_spawn_system,
                spawning::passive_mob_spawn_system,
                spawning::daylight_despawn_system,
                ai::zombie_ai_system
                    .after(spawning::mob_spawn_system),
                ai::skeleton_ai_system
                    .after(spawning::mob_spawn_system),
                ai::passive_ai_system
                    .after(spawning::mob_spawn_system),
                ai::mob_movement_system
                    .after(ai::zombie_ai_system)
                    .after(ai::skeleton_ai_system)
                    .after(ai::passive_ai_system),
                projectile::projectile_movement_system
                    .after(ai::skeleton_ai_system),
                projectile::projectile_collision_system
                    .after(projectile::projectile_movement_system),
                loot::mob_death_system,
                loot::loot_drop_system
                    .after(loot::mob_death_system),
                loot::loot_pickup_system
                    .after(loot::loot_drop_system),
                loot::mob_flee_on_damage_system,
            ).run_if(in_state(GameState::InGame)));
    }
}
