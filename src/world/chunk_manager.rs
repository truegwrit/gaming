use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use std::collections::HashMap;

use super::chunk::*;
use super::meshing::build_chunk_mesh;
use super::terrain_gen::{WorldSeed, generate_chunk};
use crate::player::controller::Player;

/// Render distance in chunks.
const RENDER_DISTANCE: i32 = 8;

/// Maps chunk coordinates to their entities.
#[derive(Resource, Default)]
pub struct ChunkMap(pub HashMap<IVec2, Entity>);

/// Task handle for async chunk generation.
#[derive(Component)]
pub struct ChunkGenTask(pub Task<(IVec2, ChunkData)>);

/// System to load/unload chunks around the player.
pub fn chunk_load_unload_system(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    chunk_map: Res<ChunkMap>,
    seed: Res<WorldSeed>,
    existing_tasks: Query<&ChunkGenTask>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_chunk = world_to_chunk(player_transform.translation);

    // Count active generation tasks
    let active_tasks = existing_tasks.iter().count();
    let max_concurrent = 4;

    let mut spawned = 0;
    // Spawn chunks in render distance
    for cx in (player_chunk.x - RENDER_DISTANCE)..=(player_chunk.x + RENDER_DISTANCE) {
        for cz in (player_chunk.y - RENDER_DISTANCE)..=(player_chunk.y + RENDER_DISTANCE) {
            let coord = IVec2::new(cx, cz);

            // Check if within circular render distance
            let dx = cx - player_chunk.x;
            let dz = cz - player_chunk.y;
            if dx * dx + dz * dz > RENDER_DISTANCE * RENDER_DISTANCE {
                continue;
            }

            if !chunk_map.0.contains_key(&coord) {
                if active_tasks + spawned >= max_concurrent {
                    return;
                }

                let seed_val = seed.0;
                let task_pool = AsyncComputeTaskPool::get();
                let task = task_pool.spawn(async move {
                    let data = generate_chunk(coord, seed_val);
                    (coord, data)
                });

                commands.spawn(ChunkGenTask(task));
                spawned += 1;
            }
        }
    }
}

/// System to poll completed chunk generation tasks and spawn chunk entities.
pub fn chunk_gen_poll_system(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tasks: Query<(Entity, &mut ChunkGenTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some((coord, chunk_data)) = block_on(future::poll_once(&mut task.0)) {
            // Remove the task entity
            commands.entity(entity).despawn();

            // Don't overwrite existing chunks
            if chunk_map.0.contains_key(&coord) {
                continue;
            }

            // Build mesh
            let mesh = build_chunk_mesh(&chunk_data);
            let mesh_handle = meshes.add(mesh);

            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.9,
                ..default()
            });

            // Spawn chunk entity
            let chunk_entity = commands
                .spawn((
                    ChunkCoord(coord),
                    chunk_data,
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        (coord.x * CHUNK_SIZE as i32) as f32,
                        0.0,
                        (coord.y * CHUNK_SIZE as i32) as f32,
                    )),
                ))
                .id();

            chunk_map.0.insert(coord, chunk_entity);
        }
    }
}

/// System to unload chunks that are too far from the player.
pub fn chunk_unload_system(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_chunk = world_to_chunk(player_transform.translation);
    let unload_distance = RENDER_DISTANCE + 2;

    let mut to_remove = Vec::new();
    for (&coord, &entity) in chunk_map.0.iter() {
        let dx = coord.x - player_chunk.x;
        let dz = coord.y - player_chunk.y;
        if dx * dx + dz * dz > unload_distance * unload_distance {
            commands.entity(entity).despawn();
            to_remove.push(coord);
        }
    }

    for coord in to_remove {
        chunk_map.0.remove(&coord);
    }
}
