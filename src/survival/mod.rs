pub mod day_night;
pub mod health;
pub mod hunger;

use bevy::prelude::*;

use crate::player::controller::Player;
use day_night::DayCycle;
use health::{DamageMessage, FallTracker, Health};
use hunger::Hunger;

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayCycle>()
            .add_message::<DamageMessage>()
            .add_systems(Startup, (
                day_night::setup_sun,
                attach_survival_components.after(crate::player::controller::spawn_player),
            ))
            .add_systems(Update, (
                day_night::advance_time_system,
                day_night::update_sun_system.after(day_night::advance_time_system),
                day_night::update_ambient_light_system.after(day_night::advance_time_system),
                day_night::update_sky_color_system.after(day_night::advance_time_system),
                hunger::hunger_depletion_system,
                hunger::starvation_system.after(hunger::hunger_depletion_system),
                health::fall_damage_system,
                health::apply_damage_system
                    .after(health::fall_damage_system)
                    .after(hunger::starvation_system),
                health::health_regen_system.after(health::apply_damage_system),
            ));
    }
}

/// Attach survival components to the player entity after it's spawned.
fn attach_survival_components(mut commands: Commands, player_q: Query<Entity, With<Player>>) {
    if let Ok(entity) = player_q.single() {
        commands.entity(entity).insert((
            Health::default(),
            Hunger::default(),
            FallTracker::default(),
        ));
    }
}
