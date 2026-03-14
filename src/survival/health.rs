use bevy::prelude::*;

use crate::player::controller::{Player, PlayerController};
use super::hunger::Hunger;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
        }
    }
}

/// Tracks player's Y position for fall damage calculation.
#[derive(Component)]
pub struct FallTracker {
    pub last_on_ground_y: f32,
    pub was_on_ground: bool,
}

impl Default for FallTracker {
    fn default() -> Self {
        Self {
            last_on_ground_y: 100.0,
            was_on_ground: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DamageSource {
    Fall,
    Starvation,
}

#[derive(Message, Clone, Debug)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
    pub source: DamageSource,
}

const REGEN_HUNGER_THRESHOLD: f32 = 18.0;
const REGEN_RATE: f32 = 0.5;

/// Detect landing and apply fall damage.
pub fn fall_damage_system(
    mut player_q: Query<(Entity, &Transform, &PlayerController, &mut FallTracker), With<Player>>,
    mut damage_writer: MessageWriter<DamageMessage>,
) {
    let Ok((entity, transform, controller, mut tracker)) = player_q.single_mut() else {
        return;
    };

    if controller.on_ground && !tracker.was_on_ground {
        // Just landed
        let fall_distance = tracker.last_on_ground_y - transform.translation.y;
        if fall_distance > 3.0 {
            let damage = fall_distance - 3.0;
            damage_writer.write(DamageMessage {
                target: entity,
                amount: damage,
                source: DamageSource::Fall,
            });
        }
    }

    if controller.on_ground {
        tracker.last_on_ground_y = transform.translation.y;
    }

    tracker.was_on_ground = controller.on_ground;
}

/// Apply damage messages to health.
pub fn apply_damage_system(
    mut damage_reader: MessageReader<DamageMessage>,
    mut health_q: Query<&mut Health>,
) {
    for msg in damage_reader.read() {
        if let Ok(mut health) = health_q.get_mut(msg.target) {
            health.current = (health.current - msg.amount).max(0.0);
        }
    }
}

/// Regenerate health when hunger is high enough.
pub fn health_regen_system(
    time: Res<Time>,
    mut player_q: Query<(&mut Health, &Hunger), With<Player>>,
) {
    let Ok((mut health, hunger)) = player_q.single_mut() else {
        return;
    };

    if hunger.current >= REGEN_HUNGER_THRESHOLD && health.current < health.max {
        health.current = (health.current + REGEN_RATE * time.delta_secs()).min(health.max);
    }
}
