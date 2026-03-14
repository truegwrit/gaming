use bevy::prelude::*;

use crate::player::controller::Player;
use crate::survival::health::{DamageMessage, DamageSource};
use crate::world::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, ChunkCoord, ChunkData};
use crate::world::chunk_manager::ChunkMap;

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const PROJECTILE_GRAVITY: f32 = -10.0;

/// A flying projectile (skeleton arrow).
#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: f32,
}

/// Spawn a projectile from origin aimed at target.
pub fn spawn_projectile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    origin: Vec3,
    target: Vec3,
    damage: f32,
) {
    let direction = (target - origin).normalize_or_zero();
    // Add slight random spread
    let spread = Vec3::new(
        (rand::random::<f32>() - 0.5) * 0.1,
        (rand::random::<f32>() - 0.5) * 0.05,
        (rand::random::<f32>() - 0.5) * 0.1,
    );
    let velocity = (direction + spread).normalize() * 20.0;

    let mesh = meshes.add(Cuboid::new(0.1, 0.1, 0.4));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.2, 0.1),
        ..default()
    });

    commands.spawn((
        Projectile {
            velocity,
            damage,
            lifetime: 5.0,
        },
        Transform::from_translation(origin).looking_to(direction, Vec3::Y),
        Mesh3d(mesh),
        MeshMaterial3d(material),
    ));
}

/// Move projectiles and apply gravity.
pub fn projectile_movement_system(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &mut Projectile)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut proj) in projectiles.iter_mut() {
        proj.velocity.y += PROJECTILE_GRAVITY * dt;
        transform.translation += proj.velocity * dt;
        proj.lifetime -= dt;

        // Update rotation to face direction of travel
        if proj.velocity.length_squared() > 0.01 {
            transform.look_to(proj.velocity.normalize(), Vec3::Y);
        }
    }
}

/// Check projectile collisions with player and blocks.
pub fn projectile_collision_system(
    mut commands: Commands,
    player_q: Query<(Entity, &Transform), With<Player>>,
    projectiles: Query<(Entity, &Transform, &Projectile)>,
    chunk_map: Res<ChunkMap>,
    chunks: Query<(&ChunkCoord, &ChunkData)>,
    mut damage_writer: MessageWriter<DamageMessage>,
) {
    let Ok((player_entity, player_tf)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;

    for (proj_entity, proj_tf, proj) in projectiles.iter() {
        let pos = proj_tf.translation;

        // Despawn if lifetime expired
        if proj.lifetime <= 0.0 {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // Check player AABB collision
        let half_w = PLAYER_WIDTH / 2.0;
        let player_min = Vec3::new(player_pos.x - half_w, player_pos.y, player_pos.z - half_w);
        let player_max = Vec3::new(player_pos.x + half_w, player_pos.y + PLAYER_HEIGHT, player_pos.z + half_w);

        if pos.x >= player_min.x && pos.x <= player_max.x
            && pos.y >= player_min.y && pos.y <= player_max.y
            && pos.z >= player_min.z && pos.z <= player_max.z
        {
            damage_writer.write(DamageMessage {
                target: player_entity,
                amount: proj.damage,
                source: DamageSource::MobProjectile,
            });
            commands.entity(proj_entity).despawn();
            continue;
        }

        // Check block collision
        let block_pos = IVec3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );

        if block_pos.y >= 0 && block_pos.y < CHUNK_HEIGHT as i32 {
            let chunk_coord = IVec2::new(
                (block_pos.x as f32 / CHUNK_SIZE as f32).floor() as i32,
                (block_pos.z as f32 / CHUNK_SIZE as f32).floor() as i32,
            );
            if let Some(&chunk_entity) = chunk_map.0.get(&chunk_coord) {
                if let Ok((_, chunk_data)) = chunks.get(chunk_entity) {
                    let lx = ((block_pos.x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
                    let lz = ((block_pos.z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
                    if chunk_data.get(lx, block_pos.y as usize, lz).is_solid() {
                        commands.entity(proj_entity).despawn();
                    }
                }
            }
        }
    }
}
