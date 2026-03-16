use bevy::prelude::*;

mod combat;
mod debug;
mod gadget;
mod inventory;
mod mobs;
mod physics;
mod player;
mod settings;
mod sound;
mod states;
mod survival;
mod ui;
mod world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RoninCraft".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<states::GameState>()
        .init_resource::<settings::GameSettings>()
        // Core game plugins
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            physics::PhysicsPlugin,
            ui::UiPlugin,
        ))
        // Feature plugins
        .add_plugins((
            gadget::GadgetPlugin,
            inventory::InventoryPlugin,
            survival::SurvivalPlugin,
            combat::CombatPlugin,
            mobs::MobPlugin,
            sound::SoundPlugin,
            debug::DebugPlugin,
        ))
        .run();
}
