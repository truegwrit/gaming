pub mod controller;

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            controller::spawn_player,
            controller::setup_lighting,
        ))
        .add_systems(Update, (
            controller::cursor_grab_system,
            controller::mouse_look_system,
            controller::player_movement_system,
        ));
    }
}
