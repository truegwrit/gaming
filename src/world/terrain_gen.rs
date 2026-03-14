use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

use super::biome::{BiomeType, biome_params, select_biome};
use super::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, ChunkData};
use super::voxel::BlockType;

#[derive(Resource)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(42)
    }
}

pub fn generate_chunk(coord: IVec2, seed: u64) -> ChunkData {
    let mut chunk = ChunkData::new_air();

    let continentalness = Perlin::new(seed as u32);
    let erosion = Perlin::new(seed as u32 + 1);
    let detail = Perlin::new(seed as u32 + 2);
    let temperature_noise = Perlin::new(seed as u32 + 10);
    let humidity_noise = Perlin::new(seed as u32 + 11);

    // Per-column biome data (stored for tree pass)
    let mut column_biomes = [[BiomeType::Plains; CHUNK_SIZE]; CHUNK_SIZE];
    let mut column_heights = [[0usize; CHUNK_SIZE]; CHUNK_SIZE];

    for local_x in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            let world_x = coord.x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = coord.y * CHUNK_SIZE as i32 + local_z as i32;

            let wx = world_x as f64;
            let wz = world_z as f64;

            // Multi-octave height sampling
            let base = continentalness.get([wx * 0.003, wz * 0.003]);
            let mid = erosion.get([wx * 0.01, wz * 0.01]) * 12.0;
            let fine = detail.get([wx * 0.05, wz * 0.05]) * 4.0;

            // Biome selection
            let temperature = temperature_noise.get([wx * 0.002, wz * 0.002]);
            let humidity = humidity_noise.get([wx * 0.002, wz * 0.002]);
            let biome = select_biome(temperature, humidity, base);
            let params = biome_params(biome);

            column_biomes[local_x][local_z] = biome;

            // Apply biome height modifiers
            let raw_height = 64.0 + base * 32.0 + mid + fine;
            let height = (raw_height * params.height_scale + params.height_offset).max(1.0) as usize;
            let height = height.min(CHUNK_HEIGHT - 1);
            column_heights[local_x][local_z] = height;

            // Fill bedrock
            chunk.set(local_x, 0, local_z, BlockType::Bedrock);

            // Fill stone
            for y in 1..height.saturating_sub(4) {
                chunk.set(local_x, y, local_z, BlockType::Stone);
            }

            // Fill subsurface layers (biome-specific)
            for y in height.saturating_sub(4)..height {
                chunk.set(local_x, y, local_z, params.subsurface_block);
            }

            // Top layer (biome-specific)
            if height > 0 {
                chunk.set(local_x, height, local_z, params.surface_block);
            }

            // Fill water up to sea level (y=60)
            let sea_level = 60;
            if height < sea_level {
                for y in (height + 1)..=sea_level {
                    if chunk.get(local_x, y, local_z) == BlockType::Air {
                        chunk.set(local_x, y, local_z, BlockType::Water);
                    }
                }
                // Sand at the bottom of water (regardless of biome)
                if height > 0 {
                    chunk.set(local_x, height, local_z, BlockType::Sand);
                }
            }

            // Mountains: snow cap above a certain height
            if biome == BiomeType::Mountains && height > 85 {
                chunk.set(local_x, height, local_z, BlockType::Snow);
            }
        }
    }

    // Generate trees (biome-aware)
    generate_trees(&mut chunk, coord, seed, &column_biomes, &column_heights);

    chunk
}

fn generate_trees(
    chunk: &mut ChunkData,
    coord: IVec2,
    seed: u64,
    biomes: &[[BiomeType; CHUNK_SIZE]; CHUNK_SIZE],
    heights: &[[usize; CHUNK_SIZE]; CHUNK_SIZE],
) {
    let tree_noise = Perlin::new(seed as u32 + 100);

    for local_x in 3..(CHUNK_SIZE - 3) {
        for local_z in 3..(CHUNK_SIZE - 3) {
            let biome = biomes[local_x][local_z];
            let params = biome_params(biome);

            if !params.tree_enabled {
                continue;
            }

            let world_x = coord.x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = coord.y * CHUNK_SIZE as i32 + local_z as i32;

            let val = tree_noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]);

            if val > params.tree_density {
                let ground_y = heights[local_x][local_z];

                if ground_y > 60 && ground_y + 7 < CHUNK_HEIGHT {
                    // Check the surface block is suitable for trees
                    let surface = chunk.get(local_x, ground_y, local_z);
                    if surface == BlockType::Grass || surface == BlockType::Snow {
                        place_tree(chunk, local_x, ground_y + 1, local_z);
                    }
                }
            }
        }
    }
}

fn place_tree(chunk: &mut ChunkData, x: usize, base_y: usize, z: usize) {
    let trunk_height = 5;

    // Trunk
    for dy in 0..trunk_height {
        chunk.set(x, base_y + dy, z, BlockType::Wood);
    }

    // Leaves (simple sphere-ish shape)
    let top = base_y + trunk_height;
    for dy in -1i32..=2 {
        let radius = if dy == 2 { 1 } else { 2 };
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx == 0 && dz == 0 && dy < 2 {
                    continue; // trunk position
                }
                let lx = x as i32 + dx;
                let ly = top as i32 + dy;
                let lz = z as i32 + dz;
                if lx >= 0
                    && lx < CHUNK_SIZE as i32
                    && ly >= 0
                    && ly < CHUNK_HEIGHT as i32
                    && lz >= 0
                    && lz < CHUNK_SIZE as i32
                {
                    let lx = lx as usize;
                    let ly = ly as usize;
                    let lz = lz as usize;
                    if chunk.get(lx, ly, lz) == BlockType::Air {
                        chunk.set(lx, ly, lz, BlockType::Leaves);
                    }
                }
            }
        }
    }
}
