use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;

use crate::settings::GameSettings;
use crate::states::GameState;

/// Tracks which state to return to when leaving the options menu.
#[derive(Resource)]
pub struct OptionsFromState(pub GameState);

impl Default for OptionsFromState {
    fn default() -> Self {
        Self(GameState::MainMenu)
    }
}

#[derive(Component)]
pub struct OptionsMenuRoot;

#[derive(Component)]
pub struct OptionsBackButton;

#[derive(Component)]
pub struct SensitivityUpButton;

#[derive(Component)]
pub struct SensitivityDownButton;

#[derive(Component)]
pub struct SensitivityValueText;

#[derive(Component)]
pub struct VolumeUpButton;

#[derive(Component)]
pub struct VolumeDownButton;

#[derive(Component)]
pub struct VolumeValueText;

pub fn setup_options_menu(mut commands: Commands) {
    commands
        .spawn((
            OptionsMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.08, 0.12)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Options"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::srgb(0.9, 0.7, 0.2)),
            ));

            parent.spawn(Node { height: Val::Px(16.0), ..default() });

            // Mouse Sensitivity row
            spawn_option_row(
                parent,
                "Mouse Sensitivity",
                "0.30",
                SensitivityDownButton,
                SensitivityUpButton,
                SensitivityValueText,
            );

            // Volume row
            spawn_option_row(
                parent,
                "Volume",
                "70%",
                VolumeDownButton,
                VolumeUpButton,
                VolumeValueText,
            );

            parent.spawn(Node { height: Val::Px(16.0), ..default() });

            // Back button
            parent
                .spawn((
                    OptionsBackButton,
                    Button,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                ))
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("Back"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn spawn_option_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    initial_value: &str,
    down_marker: impl Component,
    up_marker: impl Component,
    value_marker: impl Component,
) {
    parent
        .spawn(Node {
            width: Val::Px(400.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row: &mut ChildSpawnerCommands| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Controls: [ - ] value [ + ]
            row.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            }).with_children(|controls: &mut ChildSpawnerCommands| {
                // Minus button
                spawn_small_button(controls, "-", down_marker);

                // Value display
                controls.spawn((
                    value_marker,
                    Text::new(initial_value),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    Node {
                        width: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ));

                // Plus button
                spawn_small_button(controls, "+", up_marker);
            });
        });
}

fn spawn_small_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(30.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn update_options_visibility(
    state: Res<State<GameState>>,
    mut root_q: Query<&mut Visibility, With<OptionsMenuRoot>>,
) {
    let Ok(mut vis) = root_q.single_mut() else { return };
    *vis = if *state.get() == GameState::Options {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub fn options_button_system(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    options_from: Res<OptionsFromState>,
    mut settings: ResMut<GameSettings>,
    back_q: Query<&Interaction, (Changed<Interaction>, With<OptionsBackButton>)>,
    sens_up_q: Query<&Interaction, (Changed<Interaction>, With<SensitivityUpButton>)>,
    sens_down_q: Query<&Interaction, (Changed<Interaction>, With<SensitivityDownButton>)>,
    vol_up_q: Query<&Interaction, (Changed<Interaction>, With<VolumeUpButton>)>,
    vol_down_q: Query<&Interaction, (Changed<Interaction>, With<VolumeDownButton>)>,
    mut sens_text_q: Query<&mut Text, (With<SensitivityValueText>, Without<VolumeValueText>)>,
    mut vol_text_q: Query<&mut Text, (With<VolumeValueText>, Without<SensitivityValueText>)>,
) {
    if *state.get() != GameState::Options {
        return;
    }

    let mut changed = false;

    for interaction in sens_up_q.iter() {
        if *interaction == Interaction::Pressed {
            settings.mouse_sensitivity = (settings.mouse_sensitivity + 0.001).min(0.01);
            changed = true;
        }
    }

    for interaction in sens_down_q.iter() {
        if *interaction == Interaction::Pressed {
            settings.mouse_sensitivity = (settings.mouse_sensitivity - 0.001).max(0.001);
            changed = true;
        }
    }

    for interaction in vol_up_q.iter() {
        if *interaction == Interaction::Pressed {
            settings.master_volume = (settings.master_volume + 0.1).min(1.0);
            changed = true;
        }
    }

    for interaction in vol_down_q.iter() {
        if *interaction == Interaction::Pressed {
            settings.master_volume = (settings.master_volume - 0.1).max(0.0);
            changed = true;
        }
    }

    if changed {
        if let Ok(mut text) = sens_text_q.single_mut() {
            **text = format!("{:.2}", settings.mouse_sensitivity * 100.0);
        }
        if let Ok(mut text) = vol_text_q.single_mut() {
            **text = format!("{:.0}%", settings.master_volume * 100.0);
        }
    }

    for interaction in back_q.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(options_from.0);
        }
    }
}
