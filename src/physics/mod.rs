use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, _app: &mut App) {
        // Physics is currently handled inline in player/controller.rs
        // This plugin will be expanded with proper AABB and swept collision in Phase 2
    }
}
