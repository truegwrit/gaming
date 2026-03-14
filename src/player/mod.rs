pub mod controller;

use bevy::prelude::*;

use crate::ui::inventory_screen::InventoryScreenOpen;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, controller::spawn_player)
            .add_systems(Update, (
                controller::cursor_grab_system,
                controller::mouse_look_system
                    .run_if(|screen: Res<InventoryScreenOpen>| !screen.0),
                controller::player_movement_system,
            ));
    }
}
