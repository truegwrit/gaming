use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;

use crate::states::GameState;

#[derive(Component)]
pub struct InstructionsMenuRoot;

#[derive(Component)]
pub struct InstructionsBackButton;

pub fn setup_instructions_menu(mut commands: Commands) {
    commands
        .spawn((
            InstructionsMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.08, 0.12)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Controls"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::srgb(0.9, 0.7, 0.2)),
            ));

            parent.spawn(Node { height: Val::Px(16.0), ..default() });

            let controls = [
                ("W / A / S / D", "Move"),
                ("Mouse", "Look Around"),
                ("Left Click", "Break Block / Attack"),
                ("Right Click", "Place Block"),
                ("Space", "Jump"),
                ("Left Shift", "Sprint"),
                ("G / F", "Cycle Gadget Form"),
                ("Tab", "Open Inventory"),
                ("1 - 9", "Select Hotbar Slot"),
                ("Scroll Wheel", "Cycle Hotbar"),
                ("V", "Toggle Camera Mode"),
                ("Escape", "Pause"),
            ];

            for (key, action) in controls {
                spawn_control_row(parent, key, action);
            }

            parent.spawn(Node { height: Val::Px(16.0), ..default() });

            // Back button
            parent
                .spawn((
                    InstructionsBackButton,
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

fn spawn_control_row(parent: &mut ChildSpawnerCommands, key: &str, action: &str) {
    parent
        .spawn(Node {
            width: Val::Px(400.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(2.0)),
            ..default()
        })
        .with_children(|row: &mut ChildSpawnerCommands| {
            row.spawn((
                Text::new(key),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.3)),
            ));
            row.spawn((
                Text::new(action),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

pub fn update_instructions_visibility(
    state: Res<State<GameState>>,
    mut root_q: Query<&mut Visibility, With<InstructionsMenuRoot>>,
) {
    let Ok(mut vis) = root_q.single_mut() else { return };
    *vis = if *state.get() == GameState::Instructions {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub fn instructions_button_system(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    back_q: Query<&Interaction, (Changed<Interaction>, With<InstructionsBackButton>)>,
) {
    if *state.get() != GameState::Instructions {
        return;
    }

    for interaction in back_q.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }
}
