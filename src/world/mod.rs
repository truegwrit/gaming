pub mod biome;
pub mod block_interaction;
pub mod chunk;
pub mod chunk_manager;
pub mod meshing;
pub mod save_load;
pub mod terrain_gen;
pub mod voxel;

use bevy::prelude::*;

use block_interaction::TargetedBlock;
use chunk_manager::ChunkMap;
use save_load::ModifiedChunks;
use terrain_gen::WorldSeed;
use crate::states::GameState;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMap>()
            .init_resource::<WorldSeed>()
            .init_resource::<TargetedBlock>()
            .init_resource::<ModifiedChunks>()
            .add_message::<save_load::SaveWorldRequest>()
            .add_message::<save_load::LoadWorldRequest>()
            .add_systems(
                Update,
                (
                    chunk_manager::chunk_load_unload_system,
                    chunk_manager::chunk_gen_poll_system,
                    chunk_manager::chunk_unload_system,
                    block_interaction::update_targeted_block,
                    block_interaction::block_break_system,
                    block_interaction::block_place_system,
                    save_load::save_world_system,
                    save_load::load_world_system,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}
