use bevy::prelude::*;

#[allow(dead_code)]
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
    Paused,
}
