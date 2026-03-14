use bevy::prelude::*;

use super::voxel::BlockType;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 128;
pub const TOTAL_BLOCKS: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoord(pub IVec2);

#[derive(Component)]
pub struct ChunkData {
    blocks: Box<[BlockType; TOTAL_BLOCKS]>,
}

impl ChunkData {
    pub fn new_air() -> Self {
        Self {
            blocks: Box::new([BlockType::Air; TOTAL_BLOCKS]),
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_HEIGHT && z < CHUNK_SIZE);
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        if x >= CHUNK_SIZE || y >= CHUNK_HEIGHT || z >= CHUNK_SIZE {
            return BlockType::Air;
        }
        self.blocks[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x < CHUNK_SIZE && y < CHUNK_HEIGHT && z < CHUNK_SIZE {
            self.blocks[Self::index(x, y, z)] = block;
        }
    }
}

/// Marker component for chunks that need re-meshing.
#[allow(dead_code)]
#[derive(Component)]
pub struct ChunkMeshDirty;

/// Convert world position to chunk coordinate.
pub fn world_to_chunk(pos: Vec3) -> IVec2 {
    IVec2::new(
        (pos.x / CHUNK_SIZE as f32).floor() as i32,
        (pos.z / CHUNK_SIZE as f32).floor() as i32,
    )
}

/// Convert world position to local block coordinate within a chunk.
#[allow(dead_code)]
pub fn world_to_local(pos: Vec3) -> (usize, usize, usize) {
    let x = ((pos.x % CHUNK_SIZE as f32) + CHUNK_SIZE as f32) as usize % CHUNK_SIZE;
    let y = pos.y.max(0.0) as usize;
    let z = ((pos.z % CHUNK_SIZE as f32) + CHUNK_SIZE as f32) as usize % CHUNK_SIZE;
    (x, y.min(CHUNK_HEIGHT - 1), z)
}
