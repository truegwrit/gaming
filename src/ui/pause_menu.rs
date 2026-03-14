use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::states::GameState;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct SaveWorldButton;

#[derive(Component)]
pub struct QuitToMenuButton;

pub fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            spawn_pause_button(parent, "Resume", ResumeButton);
            spawn_pause_button(parent, "Save World", SaveWorldButton);
            spawn_pause_button(parent, "Quit to Menu", QuitToMenuButton);
        });
}

fn spawn_pause_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(180.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Toggle pause with Escape key.
pub fn toggle_pause_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    match state.get() {
        GameState::InGame => {
            next_state.set(GameState::Paused);
            if let Ok(mut cursor) = cursor_q.single_mut() {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
        }
        GameState::Paused => {
            next_state.set(GameState::InGame);
            if let Ok(mut cursor) = cursor_q.single_mut() {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
        _ => {}
    }
}

pub fn update_pause_menu_visibility(
    state: Res<State<GameState>>,
    mut root_q: Query<&mut Visibility, With<PauseMenuRoot>>,
) {
    let Ok(mut vis) = root_q.single_mut() else { return };
    *vis = if *state.get() == GameState::Paused {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub fn pause_button_system(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    resume_q: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    save_q: Query<&Interaction, (Changed<Interaction>, With<SaveWorldButton>)>,
    quit_q: Query<&Interaction, (Changed<Interaction>, With<QuitToMenuButton>)>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut save_writer: MessageWriter<crate::world::save_load::SaveWorldRequest>,
) {
    if *state.get() != GameState::Paused {
        return;
    }

    for interaction in resume_q.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::InGame);
            if let Ok(mut cursor) = cursor_q.single_mut() {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
    }

    for interaction in save_q.iter() {
        if *interaction == Interaction::Pressed {
            save_writer.write(crate::world::save_load::SaveWorldRequest);
        }
    }

    for interaction in quit_q.iter() {
        if *interaction == Interaction::Pressed {
            // Auto-save before quitting
            save_writer.write(crate::world::save_load::SaveWorldRequest);
            next_state.set(GameState::MainMenu);
        }
    }
}
