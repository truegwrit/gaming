use bevy::prelude::*;

use super::gadget::ActiveGadget;
use crate::player::animation::ParticleLifetime;
use crate::player::controller::Player;
use crate::ui::inventory_screen::InventoryScreenOpen;

/// Cycle gadget form with G (forward) and F (backward).
pub fn gadget_switch_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gadget: ResMut<ActiveGadget>,
    screen_open: Res<InventoryScreenOpen>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if screen_open.0 {
        return;
    }

    let mut switched = false;

    if keyboard.just_pressed(KeyCode::KeyG) {
        gadget.form = gadget.form.next();
        switched = true;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        gadget.form = gadget.form.prev();
        switched = true;
    }

    if switched {
        // Spawn particle burst at player position
        if let Ok(player_tf) = player_q.single() {
            let origin = player_tf.translation + Vec3::new(0.3, 1.2, 0.3);
            let particle_mesh = meshes.add(Cuboid::new(0.06, 0.06, 0.06));

            // Joy-Con accent colors (red and blue)
            let colors = [
                Color::srgb(1.0, 0.2, 0.2), // Red
                Color::srgb(0.2, 0.4, 1.0), // Blue
                gadget.form.color(),          // Gadget form color
            ];

            for i in 0..18 {
                let color = colors[i % 3];
                let mat = materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.into(),
                    ..default()
                });

                let velocity = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 3.0,
                    rand::random::<f32>() * 2.0 + 1.0,
                    (rand::random::<f32>() - 0.5) * 3.0,
                );

                let lifetime = 0.4 + rand::random::<f32>() * 0.3;

                commands.spawn((
                    ParticleLifetime { timer: lifetime, max_time: lifetime },
                    Transform::from_translation(origin + velocity * 0.05),
                    Mesh3d(particle_mesh.clone()),
                    MeshMaterial3d(mat),
                    ParticleVelocity(velocity),
                ));
            }
        }
    }
}

/// Velocity for gadget particles.
#[derive(Component)]
pub struct ParticleVelocity(pub Vec3);

/// Re-export the particle decay system and add velocity movement.
pub fn particle_decay_system(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut ParticleLifetime, Option<&ParticleVelocity>)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, velocity) in particles.iter_mut() {
        particle.timer -= dt;
        if particle.timer <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Apply velocity
        if let Some(vel) = velocity {
            transform.translation += vel.0 * dt;
        }

        // Shrink over lifetime
        let scale = (particle.timer / particle.max_time).max(0.0);
        transform.scale = Vec3::splat(scale);
    }
}
