use bevy::prelude::*;

use crate::inventory::hotbar::HotbarSelection;
use crate::inventory::inventory::Inventory;
use crate::inventory::item::ItemType;
use crate::player::controller::Player;

#[derive(Component)]
pub struct HotbarSlotUI(pub usize);

#[derive(Component)]
pub struct HotbarSlotCount(pub usize);

/// Spawn the hotbar UI at the bottom of the screen.
pub fn setup_hotbar_ui(mut commands: Commands) {
    // Hotbar container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-((9.0 * 40.0 + 8.0 * 4.0) / 2.0)),
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 0..9 {
                parent
                    .spawn((
                        HotbarSlotUI(i),
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::End,
                            align_items: AlignItems::End,
                            padding: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 0.8)),
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            HotbarSlotCount(i),
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

/// Update hotbar UI to reflect inventory contents and selection.
pub fn update_hotbar_ui_system(
    hotbar: Res<HotbarSelection>,
    player_q: Query<&Inventory, With<Player>>,
    mut slot_q: Query<(&HotbarSlotUI, &mut BackgroundColor, &mut BorderColor)>,
    mut count_q: Query<(&HotbarSlotCount, &mut Text)>,
) {
    let Ok(inventory) = player_q.single() else {
        return;
    };

    for (slot, mut bg, mut border) in slot_q.iter_mut() {
        let i = slot.0;
        let is_selected = i == hotbar.selected;

        // Border highlight for selected slot
        *border = if is_selected {
            BorderColor::all(Color::WHITE)
        } else {
            BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 0.8))
        };

        // Background tinted by item color
        if let Some(stack) = &inventory.slots[i] {
            let item_color = match stack.item {
                ItemType::Block(bt) => bt.color().to_srgba(),
                ItemType::Food(_) => Color::srgb(0.8, 0.4, 0.3).to_srgba(),
            };
            *bg = BackgroundColor(Color::srgba(
                item_color.red * 0.5,
                item_color.green * 0.5,
                item_color.blue * 0.5,
                0.8,
            ));
        } else {
            *bg = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7));
        }
    }

    for (count, mut text) in count_q.iter_mut() {
        let i = count.0;
        if let Some(stack) = &inventory.slots[i] {
            if stack.count > 1 {
                *text = Text::new(format!("{}", stack.count));
            } else {
                *text = Text::new("");
            }
        } else {
            *text = Text::new("");
        }
    }
}
