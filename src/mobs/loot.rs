use bevy::prelude::*;

use crate::inventory::inventory::Inventory;
use crate::inventory::item::{FoodType, ItemStack, ItemType};
use crate::player::controller::Player;
use crate::survival::health::Health;

use super::components::{Mob, MobAiState, MobType};

/// Floating loot drop in the world.
#[derive(Component)]
pub struct LootDrop {
    pub item: ItemStack,
    pub lifetime: f32,
    pub bob_phase: f32,
}

const LOOT_PICKUP_RANGE: f32 = 2.0;
const LOOT_LIFETIME: f32 = 300.0; // 5 minutes

/// Detect mob death (health <= 0) and spawn loot.
pub fn mob_death_system(
    mut commands: Commands,
    mob_q: Query<(Entity, &Health, &MobType, &Transform), With<Mob>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, health, mob_type, transform) in mob_q.iter() {
        if health.current > 0.0 {
            continue;
        }

        let pos = transform.translation;
        commands.entity(entity).despawn();

        // Spawn loot based on mob type
        let loot = match mob_type {
            MobType::Pig => {
                let count = 1 + (rand::random::<f32>() * 2.0) as u32;
                Some(ItemStack {
                    item: ItemType::Food(FoodType::RawPork),
                    count,
                })
            }
            // Zombie and Skeleton drop nothing for now
            _ => None,
        };

        if let Some(item) = loot {
            let mesh = meshes.add(Cuboid::new(0.3, 0.3, 0.3));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.4, 0.3),
                ..default()
            });

            commands.spawn((
                LootDrop {
                    item,
                    lifetime: LOOT_LIFETIME,
                    bob_phase: rand::random::<f32>() * std::f32::consts::TAU,
                },
                Transform::from_translation(pos + Vec3::Y * 0.5),
                Mesh3d(mesh),
                MeshMaterial3d(material),
            ));
        }
    }
}

/// Animate loot drops (bobbing) and despawn expired ones.
pub fn loot_drop_system(
    time: Res<Time>,
    mut commands: Commands,
    mut loot_q: Query<(Entity, &mut Transform, &mut LootDrop)>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut transform, mut loot) in loot_q.iter_mut() {
        loot.lifetime -= dt;
        if loot.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Bobbing animation
        let bob = (t * 2.0 + loot.bob_phase).sin() * 0.1;
        transform.translation.y += bob * dt * 2.0;

        // Slow rotation
        transform.rotate_y(dt * 1.5);
    }
}

/// Pick up loot when player walks near.
pub fn loot_pickup_system(
    mut commands: Commands,
    player_q: Query<(&Transform, Entity), With<Player>>,
    mut inventory_q: Query<&mut Inventory, With<Player>>,
    loot_q: Query<(Entity, &Transform, &LootDrop)>,
) {
    let Ok((player_tf, _)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;

    let Ok(mut inventory) = inventory_q.single_mut() else {
        return;
    };

    for (entity, loot_tf, loot) in loot_q.iter() {
        let dist = player_pos.distance(loot_tf.translation);
        if dist < LOOT_PICKUP_RANGE {
            let remainder = inventory.try_add(loot.item);
            if remainder.is_none() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Make pigs flee when damaged (triggered by health decrease).
pub fn mob_flee_on_damage_system(
    player_q: Query<&Transform, With<Player>>,
    mut mob_q: Query<(&Health, &mut MobAiState, &Transform, &MobType), (With<Mob>, Changed<Health>)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    for (health, mut ai_state, transform, mob_type) in mob_q.iter_mut() {
        if health.current <= 0.0 {
            continue;
        }

        // Pigs flee when hit
        if *mob_type == MobType::Pig {
            let flee_dir = (transform.translation - player_tf.translation).normalize_or_zero();
            let flee_dir = Vec3::new(flee_dir.x, 0.0, flee_dir.z).normalize_or_zero();
            *ai_state = MobAiState::Flee { direction: flee_dir, timer: 3.0 };
        }
    }
}
