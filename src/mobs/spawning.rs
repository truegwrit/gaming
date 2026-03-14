use bevy::prelude::*;

use crate::player::controller::Player;
use crate::survival::day_night::DayCycle;
use crate::survival::health::Health;
use crate::world::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, ChunkCoord, ChunkData};
use crate::world::chunk_manager::ChunkMap;

use super::components::*;

const SPAWN_INTERVAL: f32 = 5.0;
const HOSTILE_SPAWN_MIN_DIST: f32 = 24.0;
const HOSTILE_SPAWN_MAX_DIST: f32 = 48.0;
const PASSIVE_SPAWN_MIN_DIST: f32 = 16.0;
const PASSIVE_SPAWN_MAX_DIST: f32 = 40.0;

/// Spawn hostile mobs at night.
pub fn mob_spawn_system(
    time: Res<Time>,
    day_cycle: Res<DayCycle>,
    mut tracker: ResMut<MobSpawnTracker>,
    player_q: Query<&Transform, With<Player>>,
    hostile_q: Query<(), (With<Mob>, With<DespawnAtDay>)>,
    chunk_map: Res<ChunkMap>,
    chunks: Query<(&ChunkCoord, &ChunkData)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    tracker.spawn_timer -= time.delta_secs();
    if tracker.spawn_timer > 0.0 {
        return;
    }
    tracker.spawn_timer = SPAWN_INTERVAL;

    // Only spawn hostiles at night (0.55 - 0.95)
    let is_night = day_cycle.time > 0.55 && day_cycle.time < 0.95;
    if !is_night {
        return;
    }

    let hostile_count = hostile_q.iter().count() as u32;
    if hostile_count >= tracker.max_hostiles {
        return;
    }

    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;

    // Pick a random spawn position around the player
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let dist = HOSTILE_SPAWN_MIN_DIST + rand::random::<f32>() * (HOSTILE_SPAWN_MAX_DIST - HOSTILE_SPAWN_MIN_DIST);
    let spawn_x = player_pos.x + angle.cos() * dist;
    let spawn_z = player_pos.z + angle.sin() * dist;

    // Find surface height at spawn position
    let Some(surface_y) = find_surface_y(spawn_x as i32, spawn_z as i32, &chunk_map, &chunks) else {
        return;
    };

    // Don't spawn underwater
    if surface_y < 61 {
        return;
    }

    let spawn_pos = Vec3::new(spawn_x, surface_y as f32 + 1.0, spawn_z);

    // Randomly pick zombie or skeleton
    let mob_type = if rand::random::<f32>() > 0.4 {
        MobType::Zombie
    } else {
        MobType::Skeleton
    };

    spawn_mob_entity(&mut commands, &mut meshes, &mut materials, mob_type, spawn_pos);
}

/// Spawn passive mobs during the day.
pub fn passive_mob_spawn_system(
    time: Res<Time>,
    day_cycle: Res<DayCycle>,
    tracker: Res<MobSpawnTracker>,
    player_q: Query<&Transform, With<Player>>,
    passive_q: Query<(), (With<Mob>, Without<DespawnAtDay>)>,
    chunk_map: Res<ChunkMap>,
    chunks: Query<(&ChunkCoord, &ChunkData)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Only check every few seconds (reuse frame timing loosely)
    if (time.elapsed_secs() % 8.0) > time.delta_secs() * 2.0 {
        return;
    }

    let is_day = day_cycle.time > 0.05 && day_cycle.time < 0.50;
    if !is_day {
        return;
    }

    let passive_count = passive_q.iter().count() as u32;
    if passive_count >= tracker.max_passives {
        return;
    }

    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;

    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let dist = PASSIVE_SPAWN_MIN_DIST + rand::random::<f32>() * (PASSIVE_SPAWN_MAX_DIST - PASSIVE_SPAWN_MIN_DIST);
    let spawn_x = player_pos.x + angle.cos() * dist;
    let spawn_z = player_pos.z + angle.sin() * dist;

    let Some(surface_y) = find_surface_y(spawn_x as i32, spawn_z as i32, &chunk_map, &chunks) else {
        return;
    };

    if surface_y < 61 {
        return;
    }

    let spawn_pos = Vec3::new(spawn_x, surface_y as f32 + 1.0, spawn_z);
    spawn_mob_entity(&mut commands, &mut meshes, &mut materials, MobType::Pig, spawn_pos);
}

/// Despawn hostile mobs at dawn.
pub fn daylight_despawn_system(
    day_cycle: Res<DayCycle>,
    hostile_q: Query<Entity, With<DespawnAtDay>>,
    mut commands: Commands,
) {
    // Dawn: time crosses from night into morning
    let is_dawn = day_cycle.time > 0.0 && day_cycle.time < 0.10;
    if !is_dawn {
        return;
    }

    for entity in hostile_q.iter() {
        commands.entity(entity).despawn();
    }
}

/// Find the top solid block Y at a given world x,z.
fn find_surface_y(
    world_x: i32,
    world_z: i32,
    chunk_map: &ChunkMap,
    chunks: &Query<(&ChunkCoord, &ChunkData)>,
) -> Option<usize> {
    let chunk_coord = IVec2::new(
        (world_x as f32 / CHUNK_SIZE as f32).floor() as i32,
        (world_z as f32 / CHUNK_SIZE as f32).floor() as i32,
    );

    let entity = chunk_map.0.get(&chunk_coord)?;
    let (_, chunk_data) = chunks.get(*entity).ok()?;

    let local_x = ((world_x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
    let local_z = ((world_z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;

    // Scan from top down to find the first solid block
    for y in (0..CHUNK_HEIGHT).rev() {
        if chunk_data.get(local_x, y, local_z).is_solid() {
            return Some(y);
        }
    }
    None
}

/// Spawn a mob entity with the appropriate visual and components.
fn spawn_mob_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mob_type: MobType,
    position: Vec3,
) {
    let (color, half_extents, health, is_hostile) = match mob_type {
        MobType::Zombie => (
            Color::srgb(0.2, 0.6, 0.2),
            Vec3::new(0.4, 0.9, 0.4),
            20.0,
            true,
        ),
        MobType::Skeleton => (
            Color::srgb(0.8, 0.8, 0.75),
            Vec3::new(0.35, 0.9, 0.35),
            20.0,
            true,
        ),
        MobType::Pig => (
            Color::srgb(0.9, 0.6, 0.6),
            Vec3::new(0.4, 0.45, 0.6),
            10.0,
            false,
        ),
    };

    let mesh = meshes.add(Cuboid::new(
        half_extents.x * 2.0,
        half_extents.y * 2.0,
        half_extents.z * 2.0,
    ));
    let material = materials.add(StandardMaterial {
        base_color: color,
        ..default()
    });

    let mut entity_cmd = commands.spawn((
        Mob,
        mob_type,
        MobAiState::default(),
        Health { current: health, max: health },
        AabbCollider { half_extents },
        MobVelocity::default(),
        Transform::from_translation(position),
        Mesh3d(mesh),
        MeshMaterial3d(material),
    ));

    if is_hostile {
        entity_cmd.insert(DespawnAtDay);
    }
}
