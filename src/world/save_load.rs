use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::gadget::gadget::{ActiveGadget, GadgetForm};
use crate::inventory::hotbar::HotbarSelection;
use crate::inventory::inventory::Inventory;
use crate::inventory::item::{FoodType, ItemStack, ItemType};
use crate::player::controller::Player;
use crate::survival::day_night::DayCycle;
use crate::survival::health::Health;
use crate::survival::hunger::Hunger;
use crate::world::voxel::BlockType;

use super::chunk::{ChunkCoord, ChunkData};
use super::chunk_manager::ChunkMap;
use super::terrain_gen::{WorldSeed, generate_chunk};

/// Message to trigger world save.
#[derive(Message, Clone, Debug)]
pub struct SaveWorldRequest;

/// Message to trigger world load.
#[derive(Message, Clone, Debug)]
pub struct LoadWorldRequest;

/// Stores modified chunks that override procedural generation.
#[derive(Resource, Default)]
pub struct ModifiedChunks(pub HashMap<IVec2, ChunkData>);

#[derive(Serialize, Deserialize)]
struct WorldSaveData {
    seed: u64,
    player_pos: [f32; 3],
    player_yaw: f32,
    health: f32,
    hunger: f32,
    saturation: f32,
    inventory_slots: Vec<SavedSlot>,
    hotbar_selected: usize,
    gadget_form: u8,
    day_time: f32,
    day_count: u32,
    modified_chunks: Vec<SavedChunk>,
}

#[derive(Serialize, Deserialize)]
struct SavedSlot {
    index: usize,
    item_tag: String,
    count: u32,
}

#[derive(Serialize, Deserialize)]
struct SavedChunk {
    coord: [i32; 2],
    blocks: Vec<u8>,
}

const SAVE_DIR: &str = "saves";
const SAVE_FILE: &str = "saves/world.ron";

pub fn has_save_file() -> bool {
    std::path::Path::new(SAVE_FILE).exists()
}

/// Save the world state to disk.
pub fn save_world_system(
    mut reader: MessageReader<SaveWorldRequest>,
    player_q: Query<(&Transform, &Health, &Hunger, &Inventory), With<Player>>,
    seed: Res<WorldSeed>,
    day_cycle: Res<DayCycle>,
    hotbar: Res<HotbarSelection>,
    gadget: Res<ActiveGadget>,
    _chunk_map: Res<ChunkMap>,
    chunks: Query<(&ChunkCoord, &ChunkData)>,
) {
    let mut should_save = false;
    for _ in reader.read() {
        should_save = true;
    }
    if !should_save {
        return;
    }

    let Ok((player_tf, health, hunger, inventory)) = player_q.single() else {
        return;
    };

    let pos = player_tf.translation;
    let (_, yaw, _) = player_tf.rotation.to_euler(EulerRot::YXZ);

    // Serialize inventory
    let mut inventory_slots = Vec::new();
    for (i, slot) in inventory.slots.iter().enumerate() {
        if let Some(stack) = slot {
            let tag = match stack.item {
                ItemType::Block(bt) => format!("block:{:?}", bt),
                ItemType::Food(ft) => format!("food:{:?}", ft),
            };
            inventory_slots.push(SavedSlot {
                index: i,
                item_tag: tag,
                count: stack.count,
            });
        }
    }

    // Find modified chunks (compare against procedural generation)
    let mut modified = Vec::new();
    for (coord, chunk_data) in chunks.iter() {
        let generated = generate_chunk(coord.0, seed.0);
        if chunk_data.differs_from(&generated) {
            modified.push(SavedChunk {
                coord: [coord.0.x, coord.0.y],
                blocks: chunk_data.blocks_as_bytes().to_vec(),
            });
        }
    }

    let save_data = WorldSaveData {
        seed: seed.0,
        player_pos: [pos.x, pos.y, pos.z],
        player_yaw: yaw,
        health: health.current,
        hunger: hunger.current,
        saturation: hunger.saturation,
        inventory_slots,
        hotbar_selected: hotbar.selected,
        gadget_form: gadget.form as u8,
        day_time: day_cycle.time,
        day_count: day_cycle.day,
        modified_chunks: modified,
    };

    // Write to file
    let _ = std::fs::create_dir_all(SAVE_DIR);
    match ron::to_string(&save_data) {
        Ok(data) => {
            if let Err(e) = std::fs::write(SAVE_FILE, data) {
                eprintln!("Failed to save world: {}", e);
            } else {
                println!("World saved ({} modified chunks)", save_data.modified_chunks.len());
            }
        }
        Err(e) => eprintln!("Failed to serialize world: {}", e),
    }
}

/// Load world state from disk.
pub fn load_world_system(
    mut reader: MessageReader<LoadWorldRequest>,
    mut player_q: Query<(&mut Transform, &mut Health, &mut Hunger, &mut Inventory), With<Player>>,
    mut seed: ResMut<WorldSeed>,
    mut day_cycle: ResMut<DayCycle>,
    mut hotbar: ResMut<HotbarSelection>,
    mut gadget: ResMut<ActiveGadget>,
    mut modified_chunks: ResMut<ModifiedChunks>,
) {
    let mut should_load = false;
    for _ in reader.read() {
        should_load = true;
    }
    if !should_load {
        return;
    }

    let data = match std::fs::read_to_string(SAVE_FILE) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read save file: {}", e);
            return;
        }
    };

    let save: WorldSaveData = match ron::from_str(&data) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse save file: {}", e);
            return;
        }
    };

    // Restore world seed
    seed.0 = save.seed;

    // Restore player state
    if let Ok((mut tf, mut health, mut hunger, mut inventory)) = player_q.single_mut() {
        tf.translation = Vec3::new(save.player_pos[0], save.player_pos[1], save.player_pos[2]);
        tf.rotation = Quat::from_rotation_y(save.player_yaw);

        health.current = save.health;
        hunger.current = save.hunger;
        hunger.saturation = save.saturation;

        // Restore inventory
        inventory.slots = [None; 36];
        for slot in &save.inventory_slots {
            if slot.index < 36 {
                if let Some(item) = parse_item_tag(&slot.item_tag) {
                    inventory.slots[slot.index] = Some(ItemStack { item, count: slot.count });
                }
            }
        }
    }

    hotbar.selected = save.hotbar_selected.min(8);
    gadget.form = match save.gadget_form {
        1 => GadgetForm::Sword,
        2 => GadgetForm::Axe,
        3 => GadgetForm::Shovel,
        _ => GadgetForm::Pickaxe,
    };
    day_cycle.time = save.day_time;
    day_cycle.day = save.day_count;

    // Populate modified chunks
    modified_chunks.0.clear();
    for saved_chunk in &save.modified_chunks {
        let coord = IVec2::new(saved_chunk.coord[0], saved_chunk.coord[1]);
        let chunk_data = ChunkData::from_bytes(&saved_chunk.blocks);
        modified_chunks.0.insert(coord, chunk_data);
    }

    println!("World loaded ({} modified chunks)", save.modified_chunks.len());
}

fn parse_item_tag(tag: &str) -> Option<ItemType> {
    if let Some(block_name) = tag.strip_prefix("block:") {
        let bt = match block_name {
            "Stone" => BlockType::Stone,
            "Dirt" => BlockType::Dirt,
            "Grass" => BlockType::Grass,
            "Sand" => BlockType::Sand,
            "Wood" => BlockType::Wood,
            "Leaves" => BlockType::Leaves,
            "Cobblestone" => BlockType::Cobblestone,
            "Planks" => BlockType::Planks,
            "Gravel" => BlockType::Gravel,
            "Iron" => BlockType::Iron,
            "Coal" => BlockType::Coal,
            "Snow" => BlockType::Snow,
            _ => return None,
        };
        Some(ItemType::Block(bt))
    } else if let Some(food_name) = tag.strip_prefix("food:") {
        let ft = match food_name {
            "RawPork" => FoodType::RawPork,
            _ => return None,
        };
        Some(ItemType::Food(ft))
    } else {
        None
    }
}
