use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSettings {
    pub mouse_sensitivity: f32,
    pub master_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.003,
            master_volume: 0.7,
        }
    }
}
