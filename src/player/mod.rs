pub mod character_model;
pub mod animation;
pub mod controller;

use bevy::prelude::*;

use crate::states::GameState;
use crate::ui::inventory_screen::InventoryScreenOpen;
use character_model::CameraMode;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraMode>()
            .init_resource::<controller::FootstepTimer>()
            .add_systems(Startup, (
                controller::spawn_player,
                character_model::spawn_character_model.after(controller::spawn_player),
            ))
            .add_systems(OnEnter(GameState::InGame), controller::grab_cursor_on_enter)
            .add_systems(Update, (
                controller::cursor_grab_system,
                controller::mouse_look_system
                    .run_if(|screen: Res<InventoryScreenOpen>| !screen.0),
                controller::player_movement_system,
                controller::update_camera_position,
                character_model::toggle_camera_mode_system,
                character_model::update_character_visibility,
                animation::animate_limbs_system,
                controller::footstep_sound_system,
            ).run_if(in_state(GameState::InGame)));
    }
}
