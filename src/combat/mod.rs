pub mod attack;
pub mod knockback;

use bevy::prelude::*;

use crate::player::controller::Player;
use crate::states::GameState;
use attack::{AttackConsumed, AttackCooldown};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttackConsumed>()
            .add_systems(Startup, attach_combat_components.after(crate::player::controller::spawn_player))
            .add_systems(Update, (
                attack::reset_attack_consumed,
                attack::attack_cooldown_tick_system
                    .after(attack::reset_attack_consumed),
                attack::attack_system
                    .after(attack::attack_cooldown_tick_system),
                knockback::apply_knockback_system
                    .after(attack::attack_system),
            ).run_if(in_state(GameState::InGame)));
    }
}

fn attach_combat_components(mut commands: Commands, player_q: Query<Entity, With<Player>>) {
    if let Ok(entity) = player_q.single() {
        commands.entity(entity).insert(AttackCooldown::default());
    }
}
