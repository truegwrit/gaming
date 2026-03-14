use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum BlockType {
    #[default]
    Air = 0,
    Stone,
    Dirt,
    Grass,
    Sand,
    Wood,
    Leaves,
    Water,
    Cobblestone,
    Planks,
    Bedrock,
    Gravel,
    Iron,
    Coal,
    Snow,
}

impl BlockType {
    pub fn is_solid(self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water)
    }

    pub fn is_transparent(self) -> bool {
        matches!(self, BlockType::Air | BlockType::Water | BlockType::Leaves)
    }

    #[allow(dead_code)]
    pub fn hardness(self) -> f32 {
        match self {
            BlockType::Air => 0.0,
            BlockType::Bedrock => f32::INFINITY,
            BlockType::Stone | BlockType::Cobblestone => 1.5,
            BlockType::Iron => 3.0,
            BlockType::Coal => 2.0,
            BlockType::Wood => 2.0,
            BlockType::Planks => 1.5,
            BlockType::Dirt | BlockType::Grass | BlockType::Sand | BlockType::Gravel => 0.5,
            BlockType::Leaves | BlockType::Snow => 0.2,
            BlockType::Water => 0.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            BlockType::Air => Color::NONE,
            BlockType::Stone => Color::srgb(0.5, 0.5, 0.5),
            BlockType::Dirt => Color::srgb(0.45, 0.3, 0.15),
            BlockType::Grass => Color::srgb(0.3, 0.6, 0.2),
            BlockType::Sand => Color::srgb(0.85, 0.8, 0.55),
            BlockType::Wood => Color::srgb(0.4, 0.25, 0.1),
            BlockType::Leaves => Color::srgb(0.2, 0.5, 0.15),
            BlockType::Water => Color::srgba(0.2, 0.3, 0.8, 0.7),
            BlockType::Cobblestone => Color::srgb(0.45, 0.45, 0.45),
            BlockType::Planks => Color::srgb(0.6, 0.45, 0.2),
            BlockType::Bedrock => Color::srgb(0.2, 0.2, 0.2),
            BlockType::Gravel => Color::srgb(0.55, 0.5, 0.5),
            BlockType::Iron => Color::srgb(0.7, 0.65, 0.6),
            BlockType::Coal => Color::srgb(0.15, 0.15, 0.15),
            BlockType::Snow => Color::srgb(0.95, 0.95, 0.95),
        }
    }
}
