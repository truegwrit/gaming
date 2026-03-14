use crate::world::voxel::BlockType;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ItemType {
    Block(BlockType),
    Food(FoodType),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FoodType {
    RawPork,
}

impl FoodType {
    /// How much hunger this food restores when eaten.
    pub fn hunger_value(self) -> f32 {
        match self {
            FoodType::RawPork => 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItemStack {
    pub item: ItemType,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item: ItemType, count: u32) -> Self {
        Self { item, count }
    }

    pub fn block(block: BlockType, count: u32) -> Self {
        Self {
            item: ItemType::Block(block),
            count,
        }
    }

    pub fn max_stack_size(&self) -> u32 {
        64
    }
}

/// What item(s) a block drops when broken.
pub fn block_drop(block: BlockType) -> Option<ItemStack> {
    match block {
        BlockType::Air | BlockType::Water | BlockType::Bedrock => None,
        BlockType::Leaves => None,
        BlockType::Stone => Some(ItemStack::block(BlockType::Cobblestone, 1)),
        BlockType::Grass => Some(ItemStack::block(BlockType::Dirt, 1)),
        other => Some(ItemStack::block(other, 1)),
    }
}
