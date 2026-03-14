pub mod crafting;
pub mod hotbar;
pub mod inventory;
pub mod item;

use bevy::prelude::*;

use crate::player::controller::Player;
use crate::world::voxel::BlockType;
use crafting::RecipeRegistry;
use hotbar::HotbarSelection;
use inventory::Inventory;
use item::ItemStack;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        let mut recipes = RecipeRegistry::default();
        recipes.register_defaults();

        app.init_resource::<HotbarSelection>()
            .insert_resource(recipes)
            .add_systems(Startup, attach_inventory.after(crate::player::controller::spawn_player))
            .add_systems(Update, hotbar::hotbar_selection_system);
    }
}

/// Attach inventory to the player entity with starter items.
fn attach_inventory(mut commands: Commands, player_q: Query<Entity, With<Player>>) {
    if let Ok(entity) = player_q.single() {
        let mut inv = Inventory::default();
        // Starting items for testing
        inv.slots[0] = Some(ItemStack::block(BlockType::Cobblestone, 64));
        inv.slots[1] = Some(ItemStack::block(BlockType::Dirt, 64));
        inv.slots[2] = Some(ItemStack::block(BlockType::Wood, 32));
        inv.slots[3] = Some(ItemStack::block(BlockType::Sand, 32));
        commands.entity(entity).insert(inv);
    }
}
