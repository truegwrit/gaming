use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

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

    for local_x in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            let world_x = coord.x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = coord.y * CHUNK_SIZE as i32 + local_z as i32;

            let wx = world_x as f64;
            let wz = world_z as f64;

            // Multi-octave height sampling
            let base = continentalness.get([wx * 0.003, wz * 0.003]) * 32.0;
            let mid = erosion.get([wx * 0.01, wz * 0.01]) * 12.0;
            let fine = detail.get([wx * 0.05, wz * 0.05]) * 4.0;

            let height = (64.0 + base + mid + fine).max(1.0) as usize;
            let height = height.min(CHUNK_HEIGHT - 1);

            // Fill bedrock
            chunk.set(local_x, 0, local_z, BlockType::Bedrock);

            // Fill stone
            for y in 1..height.saturating_sub(4) {
                chunk.set(local_x, y, local_z, BlockType::Stone);
            }

            // Fill dirt layers
            for y in height.saturating_sub(4)..height {
                chunk.set(local_x, y, local_z, BlockType::Dirt);
            }

            // Top layer is grass
            if height > 0 {
                chunk.set(local_x, height, local_z, BlockType::Grass);
            }

            // Fill water up to sea level (y=60)
            let sea_level = 60;
            if height < sea_level {
                for y in (height + 1)..=sea_level {
                    if chunk.get(local_x, y, local_z) == BlockType::Air {
                        chunk.set(local_x, y, local_z, BlockType::Water);
                    }
                }
                // Sand at the bottom of water
                if height > 0 {
                    chunk.set(local_x, height, local_z, BlockType::Sand);
                }
            }
        }
    }

    // Generate trees
    generate_trees(&mut chunk, coord, seed);

    chunk
}

fn generate_trees(chunk: &mut ChunkData, coord: IVec2, seed: u64) {
    let tree_noise = Perlin::new(seed as u32 + 100);

    for local_x in 3..(CHUNK_SIZE - 3) {
        for local_z in 3..(CHUNK_SIZE - 3) {
            let world_x = coord.x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = coord.y * CHUNK_SIZE as i32 + local_z as i32;

            let val = tree_noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]);

            if val > 0.75 {
                // Find ground level
                let mut ground_y = 0;
                for y in (0..CHUNK_HEIGHT).rev() {
                    if chunk.get(local_x, y, local_z) == BlockType::Grass {
                        ground_y = y;
                        break;
                    }
                }

                if ground_y > 60 && ground_y + 7 < CHUNK_HEIGHT {
                    place_tree(chunk, local_x, ground_y + 1, local_z);
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
