pub mod gadget_hud;
pub mod hotbar_ui;
pub mod hud;
pub mod inventory_screen;
pub mod survival_hud;

use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use inventory_screen::{CraftingGrid, CursorStack, InventoryScreenOpen};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<InventoryScreenOpen>()
            .init_resource::<CursorStack>()
            .init_resource::<CraftingGrid>()
            .add_systems(Startup, (
                hud::setup_hud,
                survival_hud::setup_survival_hud,
                hotbar_ui::setup_hotbar_ui,
                inventory_screen::setup_inventory_screen,
                gadget_hud::setup_gadget_hud,
            ))
            .add_systems(Update, (
                hud::update_debug_text,
                survival_hud::update_health_bar_system,
                survival_hud::update_hunger_bar_system,
                hotbar_ui::update_hotbar_ui_system,
                inventory_screen::toggle_inventory_system,
                inventory_screen::update_inventory_visibility_system,
                inventory_screen::update_inventory_screen_system,
                inventory_screen::inventory_click_system,
                gadget_hud::update_gadget_hud_system,
            ));
    }
}
