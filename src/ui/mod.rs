pub mod hud;

use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, hud::setup_hud)
            .add_systems(Update, hud::update_debug_text);
    }
}
