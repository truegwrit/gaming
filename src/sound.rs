use bevy::prelude::*;
use bevy::audio::Volume;
use std::collections::HashMap;

use crate::settings::GameSettings;
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
#[derive(Resource, Default)]
pub struct SoundAssets(pub HashMap<String, Handle<AudioSource>>);

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundAssets>()
            .add_message::<SoundEvent>()
            .add_systems(Startup, generate_procedural_sounds)
            .add_systems(Update,
                play_sound_system.run_if(in_state(GameState::InGame))
            );
    }
}

const SAMPLE_RATE: u32 = 22050;

/// Generate all procedural sounds at startup and store handles in SoundAssets.
fn generate_procedural_sounds(
    mut sound_assets: ResMut<SoundAssets>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    let mut add = |key: &str, samples: Vec<f32>| {
        let wav = encode_wav(&samples, SAMPLE_RATE);
        let source = AudioSource { bytes: wav.into() };
        let handle = audio_sources.add(source);
        sound_assets.0.insert(key.to_string(), handle);
    };

    add("footstep_dirt", gen_footstep_dirt());
    add("footstep_stone", gen_footstep_stone());
    add("block_break", gen_block_break());
    add("block_place", gen_block_place());
    add("gadget_switch", gen_gadget_switch());
    add("attack_swing", gen_attack_swing());
    add("mob_hurt", gen_mob_hurt());
    add("player_hurt", gen_player_hurt());
}

/// Read sound events and play audio.
fn play_sound_system(
    mut reader: MessageReader<SoundEvent>,
    sound_assets: Res<SoundAssets>,
    settings: Res<GameSettings>,
    mut commands: Commands,
) {
    for event in reader.read() {
        let key = match event {
            SoundEvent::FootstepDirt => "footstep_dirt",
            SoundEvent::FootstepStone => "footstep_stone",
            SoundEvent::BlockBreak => "block_break",
            SoundEvent::BlockPlace => "block_place",
            SoundEvent::GadgetSwitch => "gadget_switch",
            SoundEvent::AttackSwing => "attack_swing",
            SoundEvent::MobHurt => "mob_hurt",
            SoundEvent::PlayerHurt => "player_hurt",
        };

        if let Some(handle) = sound_assets.0.get(key) {
            commands.spawn((
                AudioPlayer(handle.clone()),
                PlaybackSettings::ONCE
                    .with_volume(Volume::Linear(settings.master_volume)),
            ));
        }
    }
}

// --- Procedural sound generators ---

fn gen_footstep_dirt() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.08) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.08).max(0.0);
        let noise = pseudo_noise(i) * 0.3;
        let low = (t * 120.0 * std::f32::consts::TAU).sin() * 0.2;
        samples.push((noise + low) * env * 0.5);
    }
    samples
}

fn gen_footstep_stone() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.06) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.06).max(0.0);
        let noise = pseudo_noise(i) * 0.2;
        let click = (t * 800.0 * std::f32::consts::TAU).sin() * 0.4;
        samples.push((noise + click) * env * 0.5);
    }
    samples
}

fn gen_block_break() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.15) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.15).max(0.0).powi(2);
        let freq = 300.0 - t * 1200.0; // Descending pitch
        let tone = (t * freq * std::f32::consts::TAU).sin() * 0.3;
        let noise = pseudo_noise(i) * 0.4;
        samples.push((tone + noise) * env * 0.5);
    }
    samples
}

fn gen_block_place() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.1) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.1).max(0.0);
        let thud = (t * 80.0 * std::f32::consts::TAU).sin() * 0.5;
        let click = if t < 0.01 { 0.3 } else { 0.0 };
        samples.push((thud + click) * env * 0.5);
    }
    samples
}

fn gen_gadget_switch() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.15) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.15).max(0.0);
        // Two-tone ascending beep
        let freq = if t < 0.07 { 600.0 } else { 900.0 };
        let tone = (t * freq * std::f32::consts::TAU).sin() * 0.4;
        samples.push(tone * env * 0.5);
    }
    samples
}

fn gen_attack_swing() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.2) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.2).max(0.0);
        // Swoosh: noise with ascending filter
        let noise = pseudo_noise(i);
        let filter = (t * 5.0).min(1.0); // Ramp up high freq content
        samples.push(noise * env * filter * 0.4);
    }
    samples
}

fn gen_mob_hurt() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.12) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.12).max(0.0);
        let tone = (t * 200.0 * std::f32::consts::TAU).sin() * 0.3;
        let noise = pseudo_noise(i) * 0.2;
        samples.push((tone + noise) * env * 0.5);
    }
    samples
}

fn gen_player_hurt() -> Vec<f32> {
    let len = (SAMPLE_RATE as f32 * 0.15) as usize;
    let mut samples = Vec::with_capacity(len);
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / 0.15).max(0.0);
        let tone = (t * 350.0 * std::f32::consts::TAU).sin() * 0.3;
        let noise = pseudo_noise(i) * 0.3;
        samples.push((tone + noise) * env * 0.5);
    }
    samples
}

// --- Utilities ---

/// Simple deterministic pseudo-noise from sample index.
fn pseudo_noise(i: usize) -> f32 {
    let x = (i as u32).wrapping_mul(1103515245).wrapping_add(12345);
    (x as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// Encode f32 samples as a WAV file (16-bit PCM, mono).
fn encode_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len() as u32;
    let bytes_per_sample = 2u16; // 16-bit
    let num_channels = 1u16;
    let data_size = num_samples * bytes_per_sample as u32;
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = num_channels * bytes_per_sample;
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&(bytes_per_sample * 8).to_le_bytes()); // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_val = (clamped * 32767.0) as i16;
        buf.extend_from_slice(&int_val.to_le_bytes());
    }

    buf
}
