use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::inventory::Inventory;
use crate::inventory::item::{ItemStack, ItemType};
use crate::inventory::crafting::RecipeRegistry;
use crate::player::controller::Player;

/// Whether the inventory screen is open.
#[derive(Resource, Default)]
pub struct InventoryScreenOpen(pub bool);

/// The item stack currently held by the cursor (for drag-and-drop).
#[derive(Resource, Default)]
pub struct CursorStack(pub Option<ItemStack>);

/// Marker for the root inventory screen node.
#[derive(Component)]
pub struct InventoryScreenRoot;

/// Marker for an inventory slot button. Index matches Inventory slots (0-35).
#[derive(Component)]
pub struct InventorySlotButton(pub usize);

/// Marker for a crafting grid slot (0-3 for 2x2 grid).
#[derive(Component)]
pub struct CraftingSlotButton(pub usize);

/// Marker for the crafting output slot.
#[derive(Component)]
pub struct CraftingOutputButton;

/// The 2x2 crafting grid contents.
#[derive(Resource, Default)]
pub struct CraftingGrid {
    pub slots: [Option<ItemStack>; 4],
}

/// Marker for slot item count text.
#[derive(Component)]
pub struct SlotCountText;

/// Toggle inventory screen with Tab.
pub fn toggle_inventory_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut screen_open: ResMut<InventoryScreenOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        screen_open.0 = !screen_open.0;

        if let Ok(mut cursor) = cursor_q.single_mut() {
            if screen_open.0 {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            } else {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
    }
}

/// Setup the inventory screen UI (spawned hidden).
pub fn setup_inventory_screen(mut commands: Commands) {
    commands
        .spawn((
            InventoryScreenRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Main panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(350.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("Inventory"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Crafting area (2x2 grid + output)
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(16.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|craft_area| {
                            // 2x2 grid
                            craft_area
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(2.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|grid| {
                                    for row in 0..2 {
                                        grid.spawn((
                                            Node {
                                                flex_direction: FlexDirection::Row,
                                                column_gap: Val::Px(2.0),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|grid_row| {
                                            for col in 0..2 {
                                                spawn_slot(grid_row, SlotType::Crafting(row * 2 + col));
                                            }
                                        });
                                    }
                                });

                            // Arrow
                            craft_area.spawn((
                                Text::new("=>"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // Output slot
                            spawn_slot(craft_area, SlotType::CraftOutput);
                        });

                    // Storage grid (27 slots = 9x3)
                    panel.spawn((
                        Text::new("Storage"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));

                    for row in 0..3 {
                        panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(2.0),
                                    ..default()
                                },
                            ))
                            .with_children(|grid_row| {
                                for col in 0..9 {
                                    let slot_index = 9 + row * 9 + col; // slots 9-35
                                    spawn_slot(grid_row, SlotType::Inventory(slot_index));
                                }
                            });
                    }

                    // Hotbar row
                    panel.spawn((
                        Text::new("Hotbar"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));

                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(2.0),
                                ..default()
                            },
                        ))
                        .with_children(|hotbar_row| {
                            for i in 0..9 {
                                spawn_slot(hotbar_row, SlotType::Inventory(i));
                            }
                        });
                });
        });
}

enum SlotType {
    Inventory(usize),
    Crafting(usize),
    CraftOutput,
}

fn spawn_slot(parent: &mut ChildSpawnerCommands, slot_type: SlotType) {
    let mut cmd = parent.spawn((
        Button,
        Node {
            width: Val::Px(36.0),
            height: Val::Px(36.0),
            border: UiRect::all(Val::Px(1.0)),
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            padding: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 0.8)),
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
    ));

    match slot_type {
        SlotType::Inventory(idx) => { cmd.insert(InventorySlotButton(idx)); }
        SlotType::Crafting(idx) => { cmd.insert(CraftingSlotButton(idx)); }
        SlotType::CraftOutput => { cmd.insert(CraftingOutputButton); }
    }

    cmd.with_children(|slot: &mut ChildSpawnerCommands| {
        slot.spawn((
            SlotCountText,
            Text::new(""),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

/// Show/hide the inventory screen based on InventoryScreenOpen.
pub fn update_inventory_visibility_system(
    screen_open: Res<InventoryScreenOpen>,
    mut root_q: Query<&mut Visibility, With<InventoryScreenRoot>>,
) {
    let Ok(mut vis) = root_q.single_mut() else {
        return;
    };
    *vis = if screen_open.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

/// Update inventory slot visuals.
pub fn update_inventory_screen_system(
    screen_open: Res<InventoryScreenOpen>,
    player_q: Query<&Inventory, With<Player>>,
    mut slot_q: Query<(&InventorySlotButton, &mut BackgroundColor, &Children)>,
    crafting_grid: Res<CraftingGrid>,
    mut craft_slot_q: Query<(&CraftingSlotButton, &mut BackgroundColor, &Children), Without<InventorySlotButton>>,
    mut text_q: Query<&mut Text, With<SlotCountText>>,
) {
    if !screen_open.0 {
        return;
    }

    let Ok(inventory) = player_q.single() else {
        return;
    };

    // Update inventory slots
    for (slot, mut bg, children) in slot_q.iter_mut() {
        let item = &inventory.slots[slot.0];
        update_slot_visual(item, &mut bg, children, &mut text_q);
    }

    // Update crafting grid slots
    for (slot, mut bg, children) in craft_slot_q.iter_mut() {
        let item = &crafting_grid.slots[slot.0];
        update_slot_visual(item, &mut bg, children, &mut text_q);
    }
}

fn update_slot_visual(
    item: &Option<ItemStack>,
    bg: &mut BackgroundColor,
    children: &Children,
    text_q: &mut Query<&mut Text, With<SlotCountText>>,
) {
    if let Some(stack) = item {
        let c = match stack.item {
            ItemType::Block(bt) => bt.color().to_srgba(),
        };
        *bg = BackgroundColor(Color::srgba(c.red * 0.6, c.green * 0.6, c.blue * 0.6, 0.9));

        for child in children.iter() {
            if let Ok(mut text) = text_q.get_mut(child) {
                *text = if stack.count > 1 {
                    Text::new(format!("{}", stack.count))
                } else {
                    Text::new("")
                };
            }
        }
    } else {
        *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9));
        for child in children.iter() {
            if let Ok(mut text) = text_q.get_mut(child) {
                *text = Text::new("");
            }
        }
    }
}

/// Handle click interactions on inventory slots.
pub fn inventory_click_system(
    screen_open: Res<InventoryScreenOpen>,
    mut player_q: Query<&mut Inventory, With<Player>>,
    mut cursor_stack: ResMut<CursorStack>,
    mut crafting_grid: ResMut<CraftingGrid>,
    recipe_registry: Res<RecipeRegistry>,
    inv_slots: Query<(&InventorySlotButton, &Interaction), Changed<Interaction>>,
    craft_slots: Query<(&CraftingSlotButton, &Interaction), (Changed<Interaction>, Without<InventorySlotButton>)>,
    output_slots: Query<(&CraftingOutputButton, &Interaction), (Changed<Interaction>, Without<InventorySlotButton>, Without<CraftingSlotButton>)>,
) {
    if !screen_open.0 {
        return;
    }

    let Ok(mut inventory) = player_q.single_mut() else {
        return;
    };

    // Inventory slot clicks
    for (slot, interaction) in inv_slots.iter() {
        if *interaction == Interaction::Pressed {
            // Swap cursor stack with slot contents
            let temp = inventory.slots[slot.0].take();
            inventory.slots[slot.0] = cursor_stack.0.take();
            cursor_stack.0 = temp;
        }
    }

    // Crafting grid slot clicks
    for (slot, interaction) in craft_slots.iter() {
        if *interaction == Interaction::Pressed {
            let temp = crafting_grid.slots[slot.0].take();
            crafting_grid.slots[slot.0] = cursor_stack.0.take();
            cursor_stack.0 = temp;
        }
    }

    // Crafting output slot click
    for (_output, interaction) in output_slots.iter() {
        if *interaction == Interaction::Pressed {
            // Check for recipe match
            let grid = [
                [
                    crafting_grid.slots[0].map(|s| s.item),
                    crafting_grid.slots[1].map(|s| s.item),
                ],
                [
                    crafting_grid.slots[2].map(|s| s.item),
                    crafting_grid.slots[3].map(|s| s.item),
                ],
            ];

            if let Some(recipe) = recipe_registry.find_match(&grid) {
                let result = recipe.result;

                // Only craft if cursor is empty or same item with space
                let can_craft = match &cursor_stack.0 {
                    None => true,
                    Some(existing) => {
                        existing.item == result.item
                            && existing.count + result.count <= existing.max_stack_size()
                    }
                };

                if can_craft {
                    // Consume one of each input
                    for slot in crafting_grid.slots.iter_mut() {
                        if let Some(stack) = slot {
                            stack.count -= 1;
                            if stack.count == 0 {
                                *slot = None;
                            }
                        }
                    }

                    // Add result to cursor
                    if let Some(existing) = &mut cursor_stack.0 {
                        existing.count += result.count;
                    } else {
                        cursor_stack.0 = Some(result);
                    }
                }
            }
        }
    }
}
