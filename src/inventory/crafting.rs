use bevy::prelude::*;

use super::item::{ItemStack, ItemType};
use crate::world::voxel::BlockType;

#[derive(Clone)]
pub struct Recipe {
    /// 2x2 crafting grid pattern. None = empty slot.
    pub grid: [[Option<ItemType>; 2]; 2],
    pub result: ItemStack,
}

#[derive(Resource, Default)]
pub struct RecipeRegistry {
    pub recipes: Vec<Recipe>,
}

impl RecipeRegistry {
    pub fn register_defaults(&mut self) {
        // 1 Wood -> 4 Planks (any position in grid)
        let wood = Some(ItemType::Block(BlockType::Wood));
        let none = None;

        // Wood in top-left
        self.recipes.push(Recipe {
            grid: [[wood, none], [none, none]],
            result: ItemStack::block(BlockType::Planks, 4),
        });
        // Wood in top-right
        self.recipes.push(Recipe {
            grid: [[none, wood], [none, none]],
            result: ItemStack::block(BlockType::Planks, 4),
        });
        // Wood in bottom-left
        self.recipes.push(Recipe {
            grid: [[none, none], [wood, none]],
            result: ItemStack::block(BlockType::Planks, 4),
        });
        // Wood in bottom-right
        self.recipes.push(Recipe {
            grid: [[none, none], [none, wood]],
            result: ItemStack::block(BlockType::Planks, 4),
        });

        // 4 Cobblestone -> 1 Stone (2x2 cobblestone)
        let cobble = Some(ItemType::Block(BlockType::Cobblestone));
        self.recipes.push(Recipe {
            grid: [[cobble, cobble], [cobble, cobble]],
            result: ItemStack::block(BlockType::Stone, 1),
        });

        // 4 Sand -> 1 Sandstone (placeholder: just makes cobblestone for now)
        let sand = Some(ItemType::Block(BlockType::Sand));
        self.recipes.push(Recipe {
            grid: [[sand, sand], [sand, sand]],
            result: ItemStack::block(BlockType::Cobblestone, 4),
        });
    }

    /// Find a recipe matching the given 2x2 crafting grid.
    pub fn find_match(&self, grid: &[[Option<ItemType>; 2]; 2]) -> Option<&Recipe> {
        self.recipes.iter().find(|recipe| {
            recipe.grid.iter().zip(grid.iter()).all(|(recipe_row, grid_row)| {
                recipe_row.iter().zip(grid_row.iter()).all(|(r, g)| r == g)
            })
        })
    }
}
