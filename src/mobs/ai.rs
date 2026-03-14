use bevy::prelude::*;

use crate::player::controller::Player;
use crate::survival::health::{DamageMessage, DamageSource};
use crate::world::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, ChunkCoord, ChunkData};
use crate::world::chunk_manager::ChunkMap;

use super::components::*;
use super::projectile::spawn_projectile;

const ZOMBIE_DETECT_RANGE: f32 = 16.0;
const ZOMBIE_ATTACK_RANGE: f32 = 2.0;
const ZOMBIE_LOSE_RANGE: f32 = 32.0;
const ZOMBIE_DAMAGE: f32 = 3.0;
const ZOMBIE_ATTACK_COOLDOWN: f32 = 1.5;
const ZOMBIE_SPEED: f32 = 3.0;

const SKELETON_DETECT_RANGE: f32 = 24.0;
const SKELETON_ATTACK_RANGE: f32 = 12.0;
const SKELETON_LOSE_RANGE: f32 = 32.0;
const SKELETON_FLEE_RANGE: f32 = 4.0;
const SKELETON_ATTACK_COOLDOWN: f32 = 2.0;
const SKELETON_SPEED: f32 = 2.5;

const PIG_SPEED: f32 = 2.0;
const WANDER_RADIUS: f32 = 8.0;
const MOB_GRAVITY: f32 = -20.0;
const MOB_JUMP_VELOCITY: f32 = 7.0;

/// Zombie AI: chase and melee attack.
pub fn zombie_ai_system(
    time: Res<Time>,
    player_q: Query<(Entity, &Transform), With<Player>>,
    mut mob_q: Query<(&mut MobAiState, &mut MobVelocity, &Transform, &MobType), With<Mob>>,
    mut damage_writer: MessageWriter<DamageMessage>,
) {
    let Ok((player_entity, player_tf)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;
    let dt = time.delta_secs();

    for (mut ai_state, mut velocity, transform, mob_type) in mob_q.iter_mut() {
        if *mob_type != MobType::Zombie {
            continue;
        }

        let pos = transform.translation;
        let dist_to_player = pos.distance(player_pos);
        let dir_to_player = (player_pos - pos).normalize_or_zero();
        let horizontal_dir = Vec3::new(dir_to_player.x, 0.0, dir_to_player.z).normalize_or_zero();

        match ai_state.as_mut() {
            MobAiState::Idle { timer } => {
                *timer -= dt;
                velocity.value.x = 0.0;
                velocity.value.z = 0.0;
                if dist_to_player < ZOMBIE_DETECT_RANGE {
                    *ai_state = MobAiState::Chase { target_entity: player_entity };
                } else if *timer <= 0.0 {
                    let wander_target = pos + random_horizontal_offset(WANDER_RADIUS);
                    *ai_state = MobAiState::Wander { target: wander_target, timer: 4.0 };
                }
            }
            MobAiState::Wander { target, timer } => {
                *timer -= dt;
                let dir = (*target - pos).normalize_or_zero();
                velocity.value.x = dir.x * ZOMBIE_SPEED * 0.5;
                velocity.value.z = dir.z * ZOMBIE_SPEED * 0.5;

                if dist_to_player < ZOMBIE_DETECT_RANGE {
                    *ai_state = MobAiState::Chase { target_entity: player_entity };
                } else if *timer <= 0.0 || pos.distance(*target) < 1.0 {
                    *ai_state = MobAiState::Idle { timer: 2.0 + rand::random::<f32>() * 3.0 };
                }
            }
            MobAiState::Chase { .. } => {
                velocity.value.x = horizontal_dir.x * ZOMBIE_SPEED;
                velocity.value.z = horizontal_dir.z * ZOMBIE_SPEED;

                if dist_to_player < ZOMBIE_ATTACK_RANGE {
                    *ai_state = MobAiState::Attack { cooldown: 0.0 };
                } else if dist_to_player > ZOMBIE_LOSE_RANGE {
                    *ai_state = MobAiState::Idle { timer: 3.0 };
                }
            }
            MobAiState::Attack { cooldown } => {
                *cooldown -= dt;
                velocity.value.x = 0.0;
                velocity.value.z = 0.0;

                if dist_to_player > ZOMBIE_ATTACK_RANGE * 1.5 {
                    *ai_state = MobAiState::Chase { target_entity: player_entity };
                } else if *cooldown <= 0.0 {
                    damage_writer.write(DamageMessage {
                        target: player_entity,
                        amount: ZOMBIE_DAMAGE,
                        source: DamageSource::MobMelee,
                    });
                    *cooldown = ZOMBIE_ATTACK_COOLDOWN;
                }
            }
            MobAiState::Flee { direction, timer } => {
                *timer -= dt;
                velocity.value.x = direction.x * ZOMBIE_SPEED;
                velocity.value.z = direction.z * ZOMBIE_SPEED;
                if *timer <= 0.0 {
                    *ai_state = MobAiState::Idle { timer: 2.0 };
                }
            }
        }
    }
}

/// Skeleton AI: ranged attacks, flee when close.
pub fn skeleton_ai_system(
    time: Res<Time>,
    player_q: Query<(Entity, &Transform), With<Player>>,
    mut mob_q: Query<(&mut MobAiState, &mut MobVelocity, &Transform, &MobType), With<Mob>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((player_entity, player_tf)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;
    let dt = time.delta_secs();

    for (mut ai_state, mut velocity, transform, mob_type) in mob_q.iter_mut() {
        if *mob_type != MobType::Skeleton {
            continue;
        }

        let pos = transform.translation;
        let dist_to_player = pos.distance(player_pos);
        let dir_to_player = (player_pos - pos).normalize_or_zero();
        let horizontal_dir = Vec3::new(dir_to_player.x, 0.0, dir_to_player.z).normalize_or_zero();

        match ai_state.as_mut() {
            MobAiState::Idle { timer } => {
                *timer -= dt;
                velocity.value.x = 0.0;
                velocity.value.z = 0.0;
                if dist_to_player < SKELETON_DETECT_RANGE {
                    *ai_state = MobAiState::Chase { target_entity: player_entity };
                } else if *timer <= 0.0 {
                    let wander_target = pos + random_horizontal_offset(WANDER_RADIUS);
                    *ai_state = MobAiState::Wander { target: wander_target, timer: 4.0 };
                }
            }
            MobAiState::Wander { target, timer } => {
                *timer -= dt;
                let dir = (*target - pos).normalize_or_zero();
                velocity.value.x = dir.x * SKELETON_SPEED * 0.5;
                velocity.value.z = dir.z * SKELETON_SPEED * 0.5;

                if dist_to_player < SKELETON_DETECT_RANGE {
                    *ai_state = MobAiState::Chase { target_entity: player_entity };
                } else if *timer <= 0.0 || pos.distance(*target) < 1.0 {
                    *ai_state = MobAiState::Idle { timer: 2.0 + rand::random::<f32>() * 3.0 };
                }
            }
            MobAiState::Chase { .. } => {
                // Flee if player is too close
                if dist_to_player < SKELETON_FLEE_RANGE {
                    let flee_dir = -horizontal_dir;
                    *ai_state = MobAiState::Flee { direction: flee_dir, timer: 1.5 };
                } else if dist_to_player < SKELETON_ATTACK_RANGE {
                    *ai_state = MobAiState::Attack { cooldown: 0.0 };
                } else {
                    velocity.value.x = horizontal_dir.x * SKELETON_SPEED;
                    velocity.value.z = horizontal_dir.z * SKELETON_SPEED;
                }

                if dist_to_player > SKELETON_LOSE_RANGE {
                    *ai_state = MobAiState::Idle { timer: 3.0 };
                }
            }
            MobAiState::Attack { cooldown } => {
                *cooldown -= dt;
                velocity.value.x = 0.0;
                velocity.value.z = 0.0;

                // Flee if player gets too close
                if dist_to_player < SKELETON_FLEE_RANGE {
                    let flee_dir = -horizontal_dir;
                    *ai_state = MobAiState::Flee { direction: flee_dir, timer: 1.5 };
                } else if dist_to_player > SKELETON_ATTACK_RANGE * 1.2 {
                    *ai_state = MobAiState::Chase { target_entity: player_entity };
                } else if *cooldown <= 0.0 {
                    // Shoot arrow
                    let arrow_origin = pos + Vec3::Y * 1.5;
                    spawn_projectile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        arrow_origin,
                        player_pos + Vec3::Y * 1.0,
                        4.0,
                    );
                    *cooldown = SKELETON_ATTACK_COOLDOWN;
                }
            }
            MobAiState::Flee { direction, timer } => {
                *timer -= dt;
                velocity.value.x = direction.x * SKELETON_SPEED * 1.5;
                velocity.value.z = direction.z * SKELETON_SPEED * 1.5;
                if *timer <= 0.0 {
                    if dist_to_player < SKELETON_ATTACK_RANGE {
                        *ai_state = MobAiState::Attack { cooldown: 0.5 };
                    } else {
                        *ai_state = MobAiState::Chase { target_entity: player_entity };
                    }
                }
            }
        }
    }
}

/// Passive mob AI (Pig): just wander, flee when hit.
pub fn passive_ai_system(
    time: Res<Time>,
    mut mob_q: Query<(&mut MobAiState, &mut MobVelocity, &Transform, &MobType), With<Mob>>,
) {
    let dt = time.delta_secs();

    for (mut ai_state, mut velocity, transform, mob_type) in mob_q.iter_mut() {
        if *mob_type != MobType::Pig {
            continue;
        }

        let pos = transform.translation;

        match ai_state.as_mut() {
            MobAiState::Idle { timer } => {
                *timer -= dt;
                velocity.value.x = 0.0;
                velocity.value.z = 0.0;
                if *timer <= 0.0 {
                    let wander_target = pos + random_horizontal_offset(WANDER_RADIUS * 0.5);
                    *ai_state = MobAiState::Wander { target: wander_target, timer: 3.0 + rand::random::<f32>() * 4.0 };
                }
            }
            MobAiState::Wander { target, timer } => {
                *timer -= dt;
                let dir = (*target - pos).normalize_or_zero();
                velocity.value.x = dir.x * PIG_SPEED * 0.5;
                velocity.value.z = dir.z * PIG_SPEED * 0.5;

                if *timer <= 0.0 || pos.distance(*target) < 1.0 {
                    *ai_state = MobAiState::Idle { timer: 3.0 + rand::random::<f32>() * 5.0 };
                }
            }
            MobAiState::Flee { direction, timer } => {
                *timer -= dt;
                velocity.value.x = direction.x * PIG_SPEED * 2.0;
                velocity.value.z = direction.z * PIG_SPEED * 2.0;
                if *timer <= 0.0 {
                    *ai_state = MobAiState::Idle { timer: 2.0 };
                }
            }
            // Pigs don't chase or attack
            _ => {
                *ai_state = MobAiState::Idle { timer: 3.0 };
            }
        }
    }
}

/// Apply MobVelocity to Transform with gravity and basic collision.
pub fn mob_movement_system(
    time: Res<Time>,
    chunk_map: Res<ChunkMap>,
    chunks: Query<(&ChunkCoord, &ChunkData)>,
    mut mob_q: Query<(&mut Transform, &mut MobVelocity, &AabbCollider), With<Mob>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut velocity, collider) in mob_q.iter_mut() {
        // Apply gravity
        velocity.value.y += MOB_GRAVITY * dt;

        let mut new_pos = transform.translation;

        // Move X
        new_pos.x += velocity.value.x * dt;
        if check_mob_collision(new_pos, collider, &chunk_map, &chunks) {
            new_pos.x = transform.translation.x;
            // Try to jump if blocked horizontally
            if velocity.value.y.abs() < 0.1 {
                velocity.value.y = MOB_JUMP_VELOCITY;
            }
        }

        // Move Z
        new_pos.z += velocity.value.z * dt;
        if check_mob_collision(new_pos, collider, &chunk_map, &chunks) {
            new_pos.z = transform.translation.z;
            if velocity.value.y.abs() < 0.1 {
                velocity.value.y = MOB_JUMP_VELOCITY;
            }
        }

        // Move Y
        new_pos.y += velocity.value.y * dt;
        if check_mob_collision(new_pos, collider, &chunk_map, &chunks) {
            if velocity.value.y < 0.0 {
                new_pos.y = new_pos.y.ceil();
            } else {
                new_pos.y = transform.translation.y;
            }
            velocity.value.y = 0.0;
        }

        // Prevent falling below world
        if new_pos.y < -10.0 {
            // Despawn mob if it falls out of world (handled by loot system checking health)
            new_pos.y = -10.0;
        }

        transform.translation = new_pos;
    }
}

/// Check collision for mob AABB against solid blocks.
fn check_mob_collision(
    pos: Vec3,
    collider: &AabbCollider,
    chunk_map: &ChunkMap,
    chunks: &Query<(&ChunkCoord, &ChunkData)>,
) -> bool {
    let min = pos - collider.half_extents;
    let max = pos + collider.half_extents;

    let min_block = IVec3::new(
        min.x.floor() as i32,
        min.y.floor() as i32,
        min.z.floor() as i32,
    );
    let max_block = IVec3::new(
        max.x.floor() as i32,
        max.y.floor() as i32,
        max.z.floor() as i32,
    );

    for by in min_block.y..=max_block.y {
        for bz in min_block.z..=max_block.z {
            for bx in min_block.x..=max_block.x {
                if by < 0 || by >= CHUNK_HEIGHT as i32 {
                    continue;
                }
                let block = get_block(IVec3::new(bx, by, bz), chunk_map, chunks);
                if block.is_solid() {
                    return true;
                }
            }
        }
    }
    false
}

fn get_block(
    world_pos: IVec3,
    chunk_map: &ChunkMap,
    chunks: &Query<(&ChunkCoord, &ChunkData)>,
) -> crate::world::voxel::BlockType {
    use crate::world::voxel::BlockType;

    if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
        return BlockType::Air;
    }

    let chunk_coord = IVec2::new(
        (world_pos.x as f32 / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.z as f32 / CHUNK_SIZE as f32).floor() as i32,
    );

    let Some(&chunk_entity) = chunk_map.0.get(&chunk_coord) else {
        return BlockType::Air;
    };

    let Ok((_, chunk_data)) = chunks.get(chunk_entity) else {
        return BlockType::Air;
    };

    let local_x = ((world_pos.x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
    let local_z = ((world_pos.z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;

    chunk_data.get(local_x, world_pos.y as usize, local_z)
}

fn random_horizontal_offset(radius: f32) -> Vec3 {
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let dist = rand::random::<f32>() * radius;
    Vec3::new(angle.cos() * dist, 0.0, angle.sin() * dist)
}
