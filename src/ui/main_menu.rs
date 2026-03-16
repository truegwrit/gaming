use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::states::GameState;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct NewWorldButton;

#[derive(Component)]
pub struct LoadWorldButton;

#[derive(Component)]
pub struct InstructionsButton;

#[derive(Component)]
pub struct OptionsButton;

pub fn setup_main_menu(mut commands: Commands) {
    commands
        .spawn((
            MainMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.08, 0.12)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("RoninCraft"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.2)),
            ));

            // Subtitle
            parent.spawn((
                Text::new("The Switch is mightier than the sword."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // New World button
            spawn_menu_button(parent, "New World", NewWorldButton);

            // Load World button
            spawn_menu_button(parent, "Load World", LoadWorldButton);

            // Instructions button
            spawn_menu_button(parent, "Instructions", InstructionsButton);

            // Options button
            spawn_menu_button(parent, "Options", OptionsButton);
        });
}

fn spawn_menu_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Component) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(45.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn update_main_menu_visibility(
    state: Res<State<GameState>>,
    mut root_q: Query<&mut Visibility, With<MainMenuRoot>>,
) {
    let Ok(mut vis) = root_q.single_mut() else { return };
    *vis = if *state.get() == GameState::MainMenu {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub fn main_menu_button_system(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    new_world_q: Query<&Interaction, (Changed<Interaction>, With<NewWorldButton>)>,
    load_world_q: Query<&Interaction, (Changed<Interaction>, With<LoadWorldButton>)>,
    instructions_q: Query<&Interaction, (Changed<Interaction>, With<InstructionsButton>)>,
    options_q: Query<&Interaction, (Changed<Interaction>, With<OptionsButton>)>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut options_from: ResMut<crate::ui::options_menu::OptionsFromState>,
) {
    if *state.get() != GameState::MainMenu {
        return;
    }

    for interaction in new_world_q.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::InGame);
            // Grab cursor for gameplay
            if let Ok(mut cursor) = cursor_q.single_mut() {
                cursor.grab_mode = CursorGrabMode::Confined;
                cursor.visible = false;
            }
        }
    }

    for interaction in load_world_q.iter() {
        if *interaction == Interaction::Pressed {
            if crate::world::save_load::has_save_file() {
                // The load system will handle restoring state
                next_state.set(GameState::InGame);
                if let Ok(mut cursor) = cursor_q.single_mut() {
                    cursor.grab_mode = CursorGrabMode::Confined;
                    cursor.visible = false;
                }
                // TODO: trigger LoadWorldRequest after entering InGame
            }
        }
    }

    for interaction in instructions_q.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Instructions);
        }
    }

    for interaction in options_q.iter() {
        if *interaction == Interaction::Pressed {
            options_from.0 = GameState::MainMenu;
            next_state.set(GameState::Options);
        }
    }
}
