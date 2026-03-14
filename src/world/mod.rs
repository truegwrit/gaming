pub mod biome;
pub mod block_interaction;
pub mod chunk;
pub mod chunk_manager;
pub mod meshing;
pub mod terrain_gen;
pub mod voxel;

use bevy::prelude::*;

use block_interaction::TargetedBlock;
use chunk_manager::ChunkMap;
use terrain_gen::WorldSeed;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMap>()
            .init_resource::<WorldSeed>()
            .init_resource::<TargetedBlock>()
            .add_systems(
                Update,
                (
                    chunk_manager::chunk_load_unload_system,
                    chunk_manager::chunk_gen_poll_system,
                    chunk_manager::chunk_unload_system,
                    block_interaction::update_targeted_block,
                    block_interaction::block_break_system,
                    block_interaction::block_place_system,
                ),
            );
    }
}
