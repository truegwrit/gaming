use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use crate::player::controller::Player;
use crate::world::block_interaction::TargetedBlock;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct DebugText;

/// Spawn HUD elements.
pub fn setup_hud(mut commands: Commands) {
    // Crosshair
    commands
        .spawn((
            Crosshair,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(2.0),
                height: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::WHITE),
        ));

    // Crosshair horizontal line
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(20.0),
            height: Val::Px(2.0),
            margin: UiRect {
                left: Val::Px(-10.0),
                top: Val::Px(-1.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
    ));

    // Crosshair vertical line
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(2.0),
            height: Val::Px(20.0),
            margin: UiRect {
                left: Val::Px(-1.0),
                top: Val::Px(-10.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
    ));

    // Debug text
    commands.spawn((
        DebugText,
        Text::new("RoninCraft"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));

    // Block highlight indicator
    commands.spawn((
        BlockHighlightText,
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.5, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(55.0),
            margin: UiRect {
                left: Val::Px(-50.0),
                ..default()
            },
            ..default()
        },
    ));
}

#[derive(Component)]
pub struct BlockHighlightText;

/// Update debug text with FPS and player position.
pub fn update_debug_text(
    diagnostics: Res<DiagnosticsStore>,
    player_q: Query<&Transform, With<Player>>,
    targeted: Res<TargetedBlock>,
    mut text_q: Query<&mut Text, With<DebugText>>,
    mut highlight_q: Query<&mut Text, (With<BlockHighlightText>, Without<DebugText>)>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    if let Ok(player_tf) = player_q.single() {
        let pos = player_tf.translation;
        if let Ok(mut text) = text_q.single_mut() {
            *text = Text::new(format!(
                "RoninCraft v0.1\nFPS: {:.0}\nPos: {:.1}, {:.1}, {:.1}",
                fps, pos.x, pos.y, pos.z
            ));
        }
    }

    if let Ok(mut text) = highlight_q.single_mut() {
        if let Some(hit) = targeted.hit_pos {
            *text = Text::new(format!("[{}, {}, {}]", hit.x, hit.y, hit.z));
        } else {
            *text = Text::new("");
        }
    }
}
