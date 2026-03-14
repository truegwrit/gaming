use bevy::prelude::*;
use std::collections::HashMap;

use crate::states::GameState;

/// Sound event types for the game.
#[derive(Message, Clone, Debug)]
pub enum SoundEvent {
    FootstepDirt,
    FootstepStone,
    BlockBreak,
    BlockPlace,
    GadgetSwitch,
    AttackSwing,
    MobHurt,
    PlayerHurt,
}

/// Maps sound event keys to audio asset handles.
/// Empty by default — populate with real audio files when available.
#[derive(Resource, Default)]
pub struct SoundAssets(pub HashMap<String, Handle<AudioSource>>);

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundAssets>()
            .add_message::<SoundEvent>()
            .add_systems(Update,
                play_sound_system.run_if(in_state(GameState::InGame))
            );
    }
}

/// Read sound events and play audio if assets are loaded.
fn play_sound_system(
    mut reader: MessageReader<SoundEvent>,
    _sound_assets: Res<SoundAssets>,
) {
    for _event in reader.read() {
        // Sound playback is a no-op until audio assets are added to SoundAssets.
        // When assets are available, look up the event type in the HashMap
        // and spawn an AudioPlayer entity.
    }
}
