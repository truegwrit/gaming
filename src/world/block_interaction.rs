use bevy::prelude::*;
use bevy::input::mouse::MouseButton;

use super::chunk::*;
use super::meshing::build_chunk_mesh;
use super::chunk_manager::ChunkMap;
use super::voxel::BlockType;
use crate::player::controller::Player;

/// Resource tracking the currently targeted block.
#[derive(Resource, Default)]
pub struct TargetedBlock {
    /// World position of the targeted block.
    pub hit_pos: Option<IVec3>,
    /// World position of the adjacent block (for placement).
    pub place_pos: Option<IVec3>,
    /// Distance to the hit.
    #[allow(dead_code)]
    pub distance: f32,
}

/// Maximum reach distance for block interaction.
const MAX_REACH: f32 = 6.0;

/// DDA voxel raycast - steps through the voxel grid along a ray.
pub fn voxel_raycast(
    origin: Vec3,
    direction: Vec3,
    max_dist: f32,
    chunk_map: &ChunkMap,
    chunks: &Query<(&ChunkCoord, &ChunkData)>,
) -> (Option<IVec3>, Option<IVec3>) {
    let dir = direction.normalize();
    if dir.length_squared() < 0.001 {
        return (None, None);
    }

    let pos = origin;
    let mut block_pos = IVec3::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    );

    let step = IVec3::new(
        if dir.x > 0.0 { 1 } else { -1 },
        if dir.y > 0.0 { 1 } else { -1 },
        if dir.z > 0.0 { 1 } else { -1 },
    );

    let t_delta = Vec3::new(
        if dir.x.abs() > 1e-10 { (1.0 / dir.x).abs() } else { f32::MAX },
        if dir.y.abs() > 1e-10 { (1.0 / dir.y).abs() } else { f32::MAX },
        if dir.z.abs() > 1e-10 { (1.0 / dir.z).abs() } else { f32::MAX },
    );

    let mut t_max = Vec3::new(
        if dir.x > 0.0 {
            ((block_pos.x as f32 + 1.0) - pos.x) * t_delta.x
        } else {
            (pos.x - block_pos.x as f32) * t_delta.x
        },
        if dir.y > 0.0 {
            ((block_pos.y as f32 + 1.0) - pos.y) * t_delta.y
        } else {
            (pos.y - block_pos.y as f32) * t_delta.y
        },
        if dir.z > 0.0 {
            ((block_pos.z as f32 + 1.0) - pos.z) * t_delta.z
        } else {
            (pos.z - block_pos.z as f32) * t_delta.z
        },
    );

    let mut prev_pos = block_pos;
    let mut _dist = 0.0;

    for _ in 0..256 {
        // Check current block
        let block = get_block_at(block_pos, chunk_map, chunks);
        if block.is_solid() {
            return (Some(block_pos), Some(prev_pos));
        }

        prev_pos = block_pos;

        // Step to next voxel boundary
        if t_max.x < t_max.y && t_max.x < t_max.z {
            _dist = t_max.x;
            block_pos.x += step.x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            _dist = t_max.y;
            block_pos.y += step.y;
            t_max.y += t_delta.y;
        } else {
            _dist = t_max.z;
            block_pos.z += step.z;
            t_max.z += t_delta.z;
        }

        if _dist > max_dist {
            break;
        }
    }

    (None, None)
}

fn get_block_at(
    world_pos: IVec3,
    chunk_map: &ChunkMap,
    chunks: &Query<(&ChunkCoord, &ChunkData)>,
) -> BlockType {
    if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
        return BlockType::Air;
    }

    let chunk_coord = IVec2::new(
        (world_pos.x as f32 / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.z as f32 / CHUNK_SIZE as f32).floor() as i32,
    );

    let Some(&chunk_entity) = chunk_map.0.get(&chunk_coord) else {
        return BlockType::Air;
    };

    let Ok((_, chunk_data)) = chunks.get(chunk_entity) else {
        return BlockType::Air;
    };

    let local_x = ((world_pos.x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
    let local_z = ((world_pos.z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;

    chunk_data.get(local_x, world_pos.y as usize, local_z)
}

/// System to update the targeted block via raycast.
pub fn update_targeted_block(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    chunk_map: Res<ChunkMap>,
    chunks: Query<(&ChunkCoord, &ChunkData)>,
    mut targeted: ResMut<TargetedBlock>,
) {
    let Ok((cam_transform, _camera)) = camera_q.single() else {
        return;
    };

    let origin = cam_transform.translation();
    let forward = cam_transform.forward().as_vec3();

    let (hit, place) = voxel_raycast(origin, forward, MAX_REACH, &chunk_map, &chunks);
    targeted.hit_pos = hit;
    targeted.place_pos = place;
}

/// System to handle block breaking (left click).
/// Now adds dropped items to the player's inventory.
pub fn block_break_system(
    mouse: Res<ButtonInput<MouseButton>>,
    targeted: Res<TargetedBlock>,
    chunk_map: Res<ChunkMap>,
    mut chunks: Query<(&ChunkCoord, &mut ChunkData)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_handles: Query<&Mesh3d>,
    mut player_q: Query<&mut crate::inventory::inventory::Inventory, With<Player>>,
    screen_open: Res<crate::ui::inventory_screen::InventoryScreenOpen>,
    gadget: Res<crate::gadget::gadget::ActiveGadget>,
    attack_consumed: Res<crate::combat::attack::AttackConsumed>,
) {
    if screen_open.0 {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Don't break blocks if the attack system consumed this click
    if attack_consumed.0 {
        return;
    }

    // Sword form can't mine blocks
    if !gadget.form.can_mine() {
        return;
    }

    let Some(hit_pos) = targeted.hit_pos else {
        return;
    };

    // Read the block type before destroying it
    let old_block = {
        let chunk_coord = IVec2::new(
            (hit_pos.x as f32 / CHUNK_SIZE as f32).floor() as i32,
            (hit_pos.z as f32 / CHUNK_SIZE as f32).floor() as i32,
        );
        if let Some(&entity) = chunk_map.0.get(&chunk_coord) {
            if let Ok((_, chunk_data)) = chunks.get(entity) {
                let lx = ((hit_pos.x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
                let lz = ((hit_pos.z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
                chunk_data.get(lx, hit_pos.y as usize, lz)
            } else {
                BlockType::Air
            }
        } else {
            BlockType::Air
        }
    };

    set_block_at(hit_pos, BlockType::Air, &chunk_map, &mut chunks, &mut meshes, &mesh_handles);

    // Add dropped item to inventory
    if let Some(drop) = crate::inventory::item::block_drop(old_block) {
        if let Ok(mut inventory) = player_q.single_mut() {
            let _ = inventory.try_add(drop);
        }
    }
}

/// System to handle block placing (right click).
/// Now consumes items from the selected hotbar slot.
pub fn block_place_system(
    mouse: Res<ButtonInput<MouseButton>>,
    targeted: Res<TargetedBlock>,
    chunk_map: Res<ChunkMap>,
    mut chunks: Query<(&ChunkCoord, &mut ChunkData)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_handles: Query<&Mesh3d>,
    player_q_transform: Query<&Transform, With<Player>>,
    mut player_q_inv: Query<&mut crate::inventory::inventory::Inventory, With<Player>>,
    hotbar: Res<crate::inventory::hotbar::HotbarSelection>,
    screen_open: Res<crate::ui::inventory_screen::InventoryScreenOpen>,
) {
    if screen_open.0 {
        return;
    }

    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(place_pos) = targeted.place_pos else {
        return;
    };

    // Don't place block where the player is standing
    if let Ok(player_tf) = player_q_transform.single() {
        let player_block = IVec3::new(
            player_tf.translation.x.floor() as i32,
            player_tf.translation.y.floor() as i32,
            player_tf.translation.z.floor() as i32,
        );
        if place_pos == player_block || place_pos == player_block + IVec3::Y {
            return;
        }
    }

    // Get block type from selected hotbar slot
    let Ok(mut inventory) = player_q_inv.single_mut() else {
        return;
    };

    let slot = hotbar.selected;
    let Some(stack) = &inventory.slots[slot] else {
        return; // Empty slot, nothing to place
    };

    let block_type = match stack.item {
        crate::inventory::item::ItemType::Block(bt) => bt,
        _ => return, // Can't place non-block items
    };

    set_block_at(place_pos, block_type, &chunk_map, &mut chunks, &mut meshes, &mesh_handles);

    // Consume one item from the slot
    inventory.remove_from_slot(slot, 1);
}

fn set_block_at(
    world_pos: IVec3,
    block: BlockType,
    chunk_map: &ChunkMap,
    chunks: &mut Query<(&ChunkCoord, &mut ChunkData)>,
    meshes: &mut Assets<Mesh>,
    mesh_handles: &Query<&Mesh3d>,
) {
    if world_pos.y < 0 || world_pos.y >= CHUNK_HEIGHT as i32 {
        return;
    }

    let chunk_coord = IVec2::new(
        (world_pos.x as f32 / CHUNK_SIZE as f32).floor() as i32,
        (world_pos.z as f32 / CHUNK_SIZE as f32).floor() as i32,
    );

    let Some(&chunk_entity) = chunk_map.0.get(&chunk_coord) else {
        return;
    };

    let local_x = ((world_pos.x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;
    let local_z = ((world_pos.z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) as usize % CHUNK_SIZE;

    // Modify block
    if let Ok((_, mut chunk_data)) = chunks.get_mut(chunk_entity) {
        chunk_data.set(local_x, world_pos.y as usize, local_z, block);

        // Rebuild mesh
        let new_mesh = build_chunk_mesh(&chunk_data);
        if let Ok(mesh_handle) = mesh_handles.get(chunk_entity) {
            let _ = meshes.insert(&mesh_handle.0, new_mesh);
        }
    }
}
