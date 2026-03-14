use bevy::prelude::*;

use super::item::ItemStack;

/// 36-slot inventory: slots 0-8 are hotbar, 9-35 are storage.
#[derive(Component)]
pub struct Inventory {
    pub slots: [Option<ItemStack>; 36],
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [None; 36],
        }
    }
}

impl Inventory {
    /// Try to add an item stack to the inventory.
    /// Returns any remainder that didn't fit.
    pub fn try_add(&mut self, mut stack: ItemStack) -> Option<ItemStack> {
        // First pass: try to merge with existing stacks of the same type
        for slot in self.slots.iter_mut() {
            if stack.count == 0 {
                return None;
            }
            if let Some(existing) = slot {
                if existing.item == stack.item {
                    let space = existing.max_stack_size() - existing.count;
                    let transfer = stack.count.min(space);
                    existing.count += transfer;
                    stack.count -= transfer;
                }
            }
        }

        // Second pass: place in empty slots
        for slot in self.slots.iter_mut() {
            if stack.count == 0 {
                return None;
            }
            if slot.is_none() {
                let transfer = stack.count.min(stack.max_stack_size());
                *slot = Some(ItemStack {
                    item: stack.item,
                    count: transfer,
                });
                stack.count -= transfer;
            }
        }

        if stack.count > 0 {
            Some(stack)
        } else {
            None
        }
    }

    /// Remove `count` items from a specific slot. Returns the removed stack.
    pub fn remove_from_slot(&mut self, slot: usize, count: u32) -> Option<ItemStack> {
        if slot >= 36 {
            return None;
        }
        let existing = self.slots[slot].as_mut()?;
        let take = count.min(existing.count);
        let result = ItemStack {
            item: existing.item,
            count: take,
        };
        existing.count -= take;
        if existing.count == 0 {
            self.slots[slot] = None;
        }
        Some(result)
    }

    /// Swap contents of two slots.
    pub fn swap_slots(&mut self, a: usize, b: usize) {
        if a < 36 && b < 36 {
            self.slots.swap(a, b);
        }
    }
}
