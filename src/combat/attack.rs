use bevy::prelude::*;
use bevy::input::mouse::MouseButton;

use crate::gadget::gadget::ActiveGadget;
use crate::mobs::components::{AabbCollider, Mob};
use crate::player::controller::Player;
use crate::survival::health::{DamageMessage, DamageSource};
use crate::ui::inventory_screen::InventoryScreenOpen;
use super::knockback::Knockback;

/// Tracks attack cooldown on the player.
#[derive(Component)]
pub struct AttackCooldown {
    pub timer: f32,
}

impl Default for AttackCooldown {
    fn default() -> Self {
        Self { timer: 0.0 }
    }
}

/// Flag resource: true if the attack system consumed this frame's left click.
#[derive(Resource, Default)]
pub struct AttackConsumed(pub bool);

/// Reset the consumed flag each frame.
pub fn reset_attack_consumed(mut consumed: ResMut<AttackConsumed>) {
    consumed.0 = false;
}

/// Tick down the attack cooldown.
pub fn attack_cooldown_tick_system(
    time: Res<Time>,
    mut player_q: Query<&mut AttackCooldown, With<Player>>,
) {
    let Ok(mut cooldown) = player_q.single_mut() else {
        return;
    };
    cooldown.timer = (cooldown.timer - time.delta_secs()).max(0.0);
}

/// Perform melee attack on left click: entity raycast against mobs.
pub fn attack_system(
    mouse: Res<ButtonInput<MouseButton>>,
    gadget: Res<ActiveGadget>,
    screen_open: Res<InventoryScreenOpen>,
    mut consumed: ResMut<AttackConsumed>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    mut player_q: Query<&mut AttackCooldown, With<Player>>,
    mob_q: Query<(Entity, &Transform, &AabbCollider), With<Mob>>,
    mut damage_writer: MessageWriter<DamageMessage>,
    mut commands: Commands,
    mut sound_writer: MessageWriter<crate::sound::SoundEvent>,
) {
    if screen_open.0 {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(mut cooldown) = player_q.single_mut() else {
        return;
    };

    if cooldown.timer > 0.0 {
        return;
    }

    let Ok(cam_tf) = camera_q.single() else {
        return;
    };

    let origin = cam_tf.translation();
    let direction = cam_tf.forward().as_vec3();
    let reach = gadget.form.attack_reach();

    sound_writer.write(crate::sound::SoundEvent::AttackSwing);

    // Entity raycast against all mobs
    if let Some((hit_entity, hit_point)) = entity_raycast(origin, direction, reach, &mob_q) {
        // Deal damage
        damage_writer.write(DamageMessage {
            target: hit_entity,
            amount: gadget.form.attack_damage(),
            source: DamageSource::PlayerAttack,
        });

        // Apply knockback
        let knockback_dir = (hit_point - origin).normalize_or_zero();
        let knockback_velocity = Vec3::new(knockback_dir.x, 0.3, knockback_dir.z).normalize() * 8.0;
        commands.entity(hit_entity).insert(Knockback {
            velocity: knockback_velocity,
            decay: 10.0,
        });

        // Consume cooldown
        cooldown.timer = gadget.form.attack_cooldown();
        consumed.0 = true;
    }
}

/// Ray-AABB intersection test against all mob entities.
/// Returns the closest hit (entity, hit_point).
fn entity_raycast(
    origin: Vec3,
    direction: Vec3,
    max_dist: f32,
    mobs: &Query<(Entity, &Transform, &AabbCollider), With<Mob>>,
) -> Option<(Entity, Vec3)> {
    let dir = direction.normalize();
    let inv_dir = Vec3::new(
        if dir.x.abs() > 1e-10 { 1.0 / dir.x } else { f32::MAX },
        if dir.y.abs() > 1e-10 { 1.0 / dir.y } else { f32::MAX },
        if dir.z.abs() > 1e-10 { 1.0 / dir.z } else { f32::MAX },
    );

    let mut closest: Option<(Entity, Vec3, f32)> = None;

    for (entity, transform, collider) in mobs.iter() {
        let center = transform.translation;
        let min = center - collider.half_extents;
        let max = center + collider.half_extents;

        // Ray-AABB intersection (slab method)
        let t1 = (min - origin) * inv_dir;
        let t2 = (max - origin) * inv_dir;

        let t_min_v = t1.min(t2);
        let t_max_v = t1.max(t2);

        let t_enter = t_min_v.x.max(t_min_v.y).max(t_min_v.z);
        let t_exit = t_max_v.x.min(t_max_v.y).min(t_max_v.z);

        if t_enter > t_exit || t_exit < 0.0 || t_enter > max_dist {
            continue;
        }

        let t = if t_enter > 0.0 { t_enter } else { t_exit };
        let hit_point = origin + dir * t;

        if closest.is_none() || t < closest.unwrap().2 {
            closest = Some((entity, hit_point, t));
        }
    }

    closest.map(|(e, p, _)| (e, p))
}
