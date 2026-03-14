use bevy::prelude::*;
use bevy::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::asset::RenderAssetUsages;

use super::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, ChunkData};
use super::voxel::BlockType;

/// Face direction for block faces.
#[derive(Clone, Copy)]
enum Face {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl Face {
    fn normal(&self) -> [f32; 3] {
        match self {
            Face::Top => [0.0, 1.0, 0.0],
            Face::Bottom => [0.0, -1.0, 0.0],
            Face::North => [0.0, 0.0, -1.0],
            Face::South => [0.0, 0.0, 1.0],
            Face::East => [1.0, 0.0, 0.0],
            Face::West => [-1.0, 0.0, 0.0],
        }
    }
}

/// Build a mesh from chunk data using simple culled meshing.
/// Each visible face becomes a quad (2 triangles).
pub fn build_chunk_mesh(chunk: &ChunkData) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);
                if block == BlockType::Air {
                    continue;
                }

                let bx = x as f32;
                let by = y as f32;
                let bz = z as f32;
                let c = block.color().to_srgba();
                let rgba = [c.red, c.green, c.blue, c.alpha];

                // Check each face - only add if neighbor is transparent
                // Top face (y+1)
                if y + 1 >= CHUNK_HEIGHT || chunk.get(x, y + 1, z).is_transparent() {
                    if !block.is_transparent() || block != chunk.get(x, y + 1, z.min(CHUNK_SIZE - 1)) {
                        // Darken/lighten faces for visual depth
                        let face_color = shade(rgba, 1.0);
                        add_face(&mut positions, &mut normals, &mut colors, &mut indices,
                            bx, by, bz, Face::Top, face_color);
                    }
                }

                // Bottom face (y-1)
                if y == 0 || chunk.get(x, y - 1, z).is_transparent() {
                    if !block.is_transparent() {
                        let face_color = shade(rgba, 0.5);
                        add_face(&mut positions, &mut normals, &mut colors, &mut indices,
                            bx, by, bz, Face::Bottom, face_color);
                    }
                }

                // North face (z-1)
                if z == 0 || chunk.get(x, y, z - 1).is_transparent() {
                    if !block.is_transparent() {
                        let face_color = shade(rgba, 0.7);
                        add_face(&mut positions, &mut normals, &mut colors, &mut indices,
                            bx, by, bz, Face::North, face_color);
                    }
                }

                // South face (z+1)
                if z + 1 >= CHUNK_SIZE || chunk.get(x, y, z + 1).is_transparent() {
                    if !block.is_transparent() {
                        let face_color = shade(rgba, 0.7);
                        add_face(&mut positions, &mut normals, &mut colors, &mut indices,
                            bx, by, bz, Face::South, face_color);
                    }
                }

                // East face (x+1)
                if x + 1 >= CHUNK_SIZE || chunk.get(x + 1, y, z).is_transparent() {
                    if !block.is_transparent() {
                        let face_color = shade(rgba, 0.8);
                        add_face(&mut positions, &mut normals, &mut colors, &mut indices,
                            bx, by, bz, Face::East, face_color);
                    }
                }

                // West face (x-1)
                if x == 0 || chunk.get(x - 1, y, z).is_transparent() {
                    if !block.is_transparent() {
                        let face_color = shade(rgba, 0.6);
                        add_face(&mut positions, &mut normals, &mut colors, &mut indices,
                            bx, by, bz, Face::West, face_color);
                    }
                }
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn shade(color: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (color[0] * factor).min(1.0),
        (color[1] * factor).min(1.0),
        (color[2] * factor).min(1.0),
        color[3],
    ]
}

fn add_face(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    x: f32,
    y: f32,
    z: f32,
    face: Face,
    color: [f32; 4],
) {
    let base = positions.len() as u32;
    let n = face.normal();

    let verts: [[f32; 3]; 4] = match face {
        Face::Top => [
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
        ],
        Face::Bottom => [
            [x, y, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y, z],
            [x, y, z],
        ],
        Face::North => [
            [x + 1.0, y, z],
            [x, y, z],
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
        ],
        Face::South => [
            [x, y, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
        ],
        Face::East => [
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
        ],
        Face::West => [
            [x, y, z],
            [x, y, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x, y + 1.0, z],
        ],
    };

    for v in &verts {
        positions.push(*v);
        normals.push(n);
        colors.push(color);
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
