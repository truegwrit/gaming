use super::voxel::BlockType;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BiomeType {
    Plains,
    Forest,
    Desert,
    Tundra,
    Mountains,
}

pub struct BiomeParams {
    pub height_scale: f64,
    pub height_offset: f64,
    pub surface_block: BlockType,
    pub subsurface_block: BlockType,
    pub tree_density: f64,
    pub tree_enabled: bool,
}

/// Select biome from temperature and humidity noise values (-1.0 to 1.0).
pub fn select_biome(temperature: f64, humidity: f64, continentalness: f64) -> BiomeType {
    // Mountains when continentalness is very high
    if continentalness > 0.6 {
        return BiomeType::Mountains;
    }

    // Temperature/humidity quadrant mapping
    if temperature < -0.3 {
        BiomeType::Tundra
    } else if temperature > 0.3 && humidity < -0.2 {
        BiomeType::Desert
    } else if humidity > 0.2 && temperature > -0.3 {
        BiomeType::Forest
    } else {
        BiomeType::Plains
    }
}

/// Get terrain generation parameters for a biome.
pub fn biome_params(biome: BiomeType) -> BiomeParams {
    match biome {
        BiomeType::Plains => BiomeParams {
            height_scale: 1.0,
            height_offset: 0.0,
            surface_block: BlockType::Grass,
            subsurface_block: BlockType::Dirt,
            tree_density: 0.80,
            tree_enabled: true,
        },
        BiomeType::Forest => BiomeParams {
            height_scale: 1.0,
            height_offset: 2.0,
            surface_block: BlockType::Grass,
            subsurface_block: BlockType::Dirt,
            tree_density: 0.50,
            tree_enabled: true,
        },
        BiomeType::Desert => BiomeParams {
            height_scale: 0.6,
            height_offset: -2.0,
            surface_block: BlockType::Sand,
            subsurface_block: BlockType::Sand,
            tree_density: 1.0, // No trees
            tree_enabled: false,
        },
        BiomeType::Tundra => BiomeParams {
            height_scale: 0.8,
            height_offset: 0.0,
            surface_block: BlockType::Snow,
            subsurface_block: BlockType::Dirt,
            tree_density: 0.90,
            tree_enabled: true,
        },
        BiomeType::Mountains => BiomeParams {
            height_scale: 2.0,
            height_offset: 20.0,
            surface_block: BlockType::Stone,
            subsurface_block: BlockType::Stone,
            tree_density: 0.90,
            tree_enabled: true,
        },
    }
}
