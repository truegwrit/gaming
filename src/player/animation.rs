use bevy::prelude::*;

use super::character_model::{BodyPart, CameraMode, CharacterLimb};
use super::controller::{Player, PlayerController};

/// Animate character limbs with sin-wave walking motion.
pub fn animate_limbs_system(
    time: Res<Time>,
    camera_mode: Res<CameraMode>,
    player_q: Query<&PlayerController, With<Player>>,
    mut limb_q: Query<(&CharacterLimb, &mut Transform)>,
) {
    // Only animate in third person (model is hidden in first person)
    if *camera_mode == CameraMode::FirstPerson {
        return;
    }

    let Ok(controller) = player_q.single() else {
        return;
    };

    let horizontal_speed = Vec3::new(controller.velocity.x, 0.0, controller.velocity.z).length();
    let is_moving = horizontal_speed > 0.5;
    let dt = time.delta_secs();

    let (frequency, amplitude) = if controller.is_sprinting && is_moving {
        (10.0, 0.8)
    } else {
        (8.0, 0.6)
    };

    let t = time.elapsed_secs();

    for (limb, mut transform) in limb_q.iter_mut() {
        let target_rotation = if is_moving {
            let swing = (t * frequency).sin() * amplitude;
            match limb.part {
                BodyPart::LeftArm => Quat::from_rotation_x(swing),
                BodyPart::RightArm => Quat::from_rotation_x(-swing),
                BodyPart::LeftLeg => Quat::from_rotation_x(-swing),
                BodyPart::RightLeg => Quat::from_rotation_x(swing),
                _ => Quat::IDENTITY,
            }
        } else {
            Quat::IDENTITY
        };

        // Smooth lerp toward target rotation
        transform.rotation = transform.rotation.slerp(target_rotation, (dt * 10.0).min(1.0));
    }
}

/// Component for decaying particles.
#[derive(Component)]
pub struct ParticleLifetime {
    pub timer: f32,
    pub max_time: f32,
}

// ParticleLifetime decay is handled by gadget::input::particle_decay_system
