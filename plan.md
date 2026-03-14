Building "RoninCraft" - a Minecraft-style voxel game in Rust/Bevy starring Ronin, a kid with dreadlocks, a green Spider-Man t-shirt, and checkered shorts. His unique mechanic: a Nintendo Switch that transforms into tools and weapons. The repo is empty (just a README), so we're starting from scratch.

Tech Stack
* Language: Rust (2024 edition)
* Engine: Bevy 0.15 (latest stable)
* Key crates: noise (terrain gen), dot_vox (voxel models), rand, serde/ron (data files)
* Meshing: Custom greedy meshing (or bevy_meshem for speed)

Project Structure
ronincraft/
├── Cargo.toml
├── .cargo/config.toml          # Fast linking (LLD)
├── assets/
│   ├── textures/blocks/        # Block face textures
│   ├── textures/ui/            # UI textures
│   └── models/                 # .vox files for Ronin + gadgets
├── src/
│   ├── main.rs                 # App entry, plugin registration
│   ├── lib.rs                  # Shared types
│   ├── states.rs               # GameState enum (Menu, Loading, InGame, Paused)
│   ├── world/
│   │   ├── mod.rs              # WorldPlugin
│   │   ├── voxel.rs            # BlockType enum, block properties
│   │   ├── chunk.rs            # ChunkData (16x256x16 block array)
│   │   ├── chunk_manager.rs    # Load/unload chunks around player
│   │   ├── terrain_gen.rs      # Noise-based procedural terrain
│   │   ├── biome.rs            # Biome selection (heat/humidity)
│   │   ├── meshing.rs          # Greedy meshing, face culling
│   │   └── block_interaction.rs # Block place/break + DDA raycast
│   ├── player/
│   │   ├── mod.rs              # PlayerPlugin
│   │   ├── controller.rs       # WASD movement, gravity, jumping
│   │   ├── camera.rs           # First/third-person camera
│   │   └── input.rs            # Input mapping
│   ├── gadget/
│   │   ├── mod.rs              # GadgetPlugin
│   │   ├── switch_device.rs    # SwitchGadget component, GadgetForm enum
│   │   ├── tools.rs            # Pickaxe/axe/shovel behaviors
│   │   └── weapons.rs          # Sword/bow/shield behaviors
│   ├── inventory/
│   │   ├── mod.rs              # InventoryPlugin
│   │   ├── inventory.rs        # 36-slot inventory + item stacks
│   │   └── crafting.rs         # Recipe registry + crafting grid
│   ├── combat/
│   │   ├── mod.rs              # CombatPlugin
│   │   ├── health.rs           # Health/damage/death
│   │   └── attack.rs           # Melee/ranged attacks
│   ├── mobs/
│   │   ├── mod.rs              # MobPlugin
│   │   ├── spawner.rs          # Spawn rules (light, biome, distance)
│   │   └── ai.rs               # State machine AI (idle/chase/attack)
│   ├── survival/
│   │   ├── mod.rs              # SurvivalPlugin
│   │   ├── hunger.rs           # Hunger depletion + effects
│   │   └── day_night.rs        # 20-min day cycle, lighting
│   ├── ui/
│   │   ├── mod.rs              # UiPlugin
│   │   ├── hud.rs              # Crosshair, hotbar, health/hunger bars
│   │   ├── inventory_screen.rs # Inventory + crafting UI
│   │   └── main_menu.rs        # Title screen
│   └── physics/
│       ├── mod.rs              # PhysicsPlugin
│       ├── aabb.rs             # Bounding box
│       └── collision.rs        # Voxel collision detection

The Switch-Gadget Mechanic (Ronin's Unique Feature)
Ronin's Nintendo Switch physically morphs into tools/weapons:
* GadgetForm enum: Switch (idle), Pickaxe, Axe, Shovel, Sword, Shield, Bow, Torch
* Transformation: Scroll wheel or number keys trigger a 0.3s morph animation with particle effects
* Visual identity: Every form retains red/blue Joy-Con color accents
* Gameplay effect: Each form modifies relevant systems (mining speed, damage, defense)
* Unlocking: Phase 1 starts with Pickaxe + Sword; more forms unlock via crafting later

Phased Implementation
Phase 1: Playable Prototype (Build First)
* Project scaffold - Cargo.toml, main.rs, Bevy app with window + state machine
* BlockType enum + ChunkData struct (16x256x16, 1 byte per block)
* Flat terrain generation (stone/dirt/grass layers) to validate pipeline
* Naive meshing (one quad per visible face) + render single chunk
* ChunkManager: load chunks in radius around player
* First-person camera with mouse look
* Player movement: WASD, gravity, ground detection
* AABB collision against voxel grid (sweep X/Y/Z independently)
* DDA voxel raycast + block highlight outline
* Block break (left click) + block place (right click) + chunk re-mesh
* Noise-based terrain (Perlin heightmap with hills/valleys)
* Async chunk generation + meshing (off main thread)
* Greedy meshing upgrade for performance
* Texture atlas for block faces
* Basic HUD: crosshair, hotbar, debug info (FPS, coords)

Phase 2: Survival Foundation
* Day/night cycle (20-min real-time cycle, lighting changes)
* Health + hunger systems
* Inventory system (36 slots) + inventory UI with drag-and-drop
* Crafting system with recipe registry
* Multiple biomes (plains, forest, desert, tundra, mountains)
* Trees and vegetation generation

Phase 3: Combat & Gadgets
* Switch-gadget system (transformation, per-form behaviors)
* Melee combat (swing, damage, knockback, cooldown)
* 2-3 hostile mob types with state-machine AI
* Passive mobs (drop items)
* Mob spawning rules (night, dark areas)
* Loot drops

Phase 4: Polish & Character
* Ronin voxel character model (MagicaVoxel .vox, segmented body parts)
* Third-person camera toggle
* Rigid-body limb animation (walking, mining, combat)
* Switch morph animation with particles
* Sound effects (footsteps, block break/place, combat)
* World save/load (serialize modified chunks)
* Main menu + pause menu

Key ECS Design Decisions
* Chunk entities: Each chunk is an entity with ChunkCoord + ChunkData + Mesh3d components
* ChunkMap resource: HashMap<IVec2, Entity> for O(1) chunk lookup
* Async tasks: Use AsyncComputeTaskPool for terrain gen + meshing
* System ordering: InputSet -> PhysicsSet -> WorldUpdateSet -> CombatSet -> UiSet (chained)
* Gadget as component: SwitchGadget component on player entity; systems query it to modify mining speed, damage, etc.

Verification
After Phase 1 implementation:
* cargo build compiles without errors
* cargo run opens a window showing a voxel world with terrain
* WASD + mouse look navigates the world
* Left-click breaks blocks, right-click places blocks
* Chunks load/unload as player moves (no lag spikes)
* FPS stays above 60 with 8-chunk render distance
