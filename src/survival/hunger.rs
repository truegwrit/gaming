use bevy::prelude::*;

use crate::player::controller::{Player, PlayerController};
use super::health::{DamageMessage, DamageSource};

const BASE_DEPLETION_RATE: f32 = 0.1;
const SPRINT_DEPLETION_MULT: f32 = 3.0;
const STARVATION_INTERVAL: f32 = 4.0;
const STARVATION_DAMAGE: f32 = 1.0;

#[derive(Component)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
    pub saturation: f32,
    pub starvation_timer: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
            saturation: 5.0,
            starvation_timer: 0.0,
        }
    }
}

/// Deplete hunger over time (faster when sprinting).
pub fn hunger_depletion_system(
    time: Res<Time>,
    mut player_q: Query<(&PlayerController, &mut Hunger), With<Player>>,
) {
    let Ok((controller, mut hunger)) = player_q.single_mut() else {
        return;
    };

    let rate = if controller.is_sprinting {
        BASE_DEPLETION_RATE * SPRINT_DEPLETION_MULT
    } else {
        BASE_DEPLETION_RATE
    };

    let drain = rate * time.delta_secs();

    // Drain saturation first, then hunger
    if hunger.saturation > 0.0 {
        hunger.saturation = (hunger.saturation - drain).max(0.0);
    } else {
        hunger.current = (hunger.current - drain).max(0.0);
    }
}

/// Deal starvation damage when hunger is at zero.
pub fn starvation_system(
    time: Res<Time>,
    mut player_q: Query<(Entity, &mut Hunger), With<Player>>,
    mut damage_writer: MessageWriter<DamageMessage>,
) {
    let Ok((entity, mut hunger)) = player_q.single_mut() else {
        return;
    };

    if hunger.current <= 0.0 {
        hunger.starvation_timer += time.delta_secs();
        if hunger.starvation_timer >= STARVATION_INTERVAL {
            hunger.starvation_timer -= STARVATION_INTERVAL;
            damage_writer.write(DamageMessage {
                target: entity,
                amount: STARVATION_DAMAGE,
                source: DamageSource::Starvation,
            });
        }
    } else {
        hunger.starvation_timer = 0.0;
    }
}
