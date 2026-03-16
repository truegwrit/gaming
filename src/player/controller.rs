use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::world::chunk::{CHUNK_HEIGHT, CHUNK_SIZE};
use crate::world::chunk_manager::ChunkMap;
use crate::world::chunk::ChunkCoord;
use crate::world::chunk::ChunkData;

/// Marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// Player physics state.
#[derive(Component)]
pub struct PlayerController {
    pub velocity: Vec3,
    pub on_ground: bool,
    pub is_sprinting: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            on_ground: false,
            is_sprinting: false,
        }
    }
}

/// Timer for throttling footstep sounds.
#[derive(Resource)]
pub struct FootstepTimer(pub f32);

impl Default for FootstepTimer {
    fn default() -> Self {
        Self(0.0)
    }
}

const GRAVITY: f32 = -20.0;
const JUMP_VELOCITY: f32 = 7.5;
const WALK_SPEED: f32 = 5.0;
const SPRINT_SPEED: f32 = 8.0;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_EYE_HEIGHT: f32 = 1.6;

/// Spawn the player entity with camera.
pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            PlayerController::default(),
            Transform::from_xyz(8.0, 100.0, 8.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 0.0),
                Projection::Perspective(PerspectiveProjection {
                    fov: 70.0_f32.to_radians(),
                    ..default()
                }),
            ));
        });
}

// Note: Lighting setup moved to survival/day_night.rs (setup_sun)

/// System to handle mouse look (camera rotation).
pub fn mouse_look_system(
    mouse_motion: Res<AccumulatedMouseMotion>,
    settings: Res<crate::settings::GameSettings>,
    mut player_q: Query<&mut Transform, With<Player>>,
    mut camera_q: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    let delta = mouse_motion.delta;

    if delta.length_squared() < 1e-6 {
        return;
    }

    let sensitivity = settings.mouse_sensitivity;

    // Yaw on the player (rotate around Y)
    if let Ok(mut player_tf) = player_q.single_mut() {
        player_tf.rotate_y(-delta.x * sensitivity);
    }

    // Pitch on the camera (rotate around local X)
    if let Ok(mut camera_tf) = camera_q.single_mut() {
        let pitch = -delta.y * sensitivity;
        camera_tf.rotate_local_x(pitch);

        // Clamp pitch to prevent flipping
        let (_, current_pitch, _) = camera_tf.rotation.to_euler(EulerRot::YXZ);
        if current_pitch.abs() > 1.5 {
            let clamped = current_pitch.clamp(-1.5, 1.5);
            camera_tf.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, clamped, 0.0);
        }
    }
}

/// System to handle keyboard movement.
pub fn player_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_q: Query<(&mut Transform, &mut PlayerController), With<Player>>,
    chunk_map: Res<ChunkMap>,
    chunks: Query<(&ChunkCoord, &ChunkData)>,
) {
    let Ok((mut transform, mut controller)) = player_q.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Sprint check
    controller.is_sprinting = keyboard.pressed(KeyCode::ShiftLeft);
    let speed = if controller.is_sprinting {
        SPRINT_SPEED
    } else {
        WALK_SPEED
    };

    // Calculate movement direction relative to player facing
    let mut move_dir = Vec3::ZERO;
    let forward = transform.forward().as_vec3();
    let right = transform.right().as_vec3();

    // Project to horizontal plane
    let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    if keyboard.pressed(KeyCode::KeyW) {
        move_dir += forward_flat;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_dir -= forward_flat;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_dir += right_flat;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        move_dir -= right_flat;
    }

    if move_dir.length_squared() > 0.0 {
        move_dir = move_dir.normalize() * speed;
    }

    // Apply horizontal velocity
    controller.velocity.x = move_dir.x;
    controller.velocity.z = move_dir.z;

    // Apply gravity
    controller.velocity.y += GRAVITY * dt;

    // Jump
    if keyboard.just_pressed(KeyCode::Space) && controller.on_ground {
        controller.velocity.y = JUMP_VELOCITY;
        controller.on_ground = false;
    }

    // Move and collide
    let velocity = controller.velocity;
    let mut new_pos = transform.translation;

    // Move along each axis independently for proper collision
    // X axis
    new_pos.x += velocity.x * dt;
    if check_collision(new_pos, &chunk_map, &chunks) {
        new_pos.x = transform.translation.x;
        controller.velocity.x = 0.0;
    }

    // Z axis
    new_pos.z += velocity.z * dt;
    if check_collision(new_pos, &chunk_map, &chunks) {
        new_pos.z = transform.translation.z;
        controller.velocity.z = 0.0;
    }

    // Y axis
    new_pos.y += velocity.y * dt;
    if check_collision(new_pos, &chunk_map, &chunks) {
        if velocity.y < 0.0 {
            controller.on_ground = true;
            new_pos.y = new_pos.y.ceil();
        } else {
            new_pos.y = transform.translation.y;
        }
        controller.velocity.y = 0.0;
    } else {
        controller.on_ground = false;
    }

    // Prevent falling below world
    if new_pos.y < -10.0 {
        new_pos = Vec3::new(8.0, 100.0, 8.0);
        controller.velocity = Vec3::ZERO;
    }

    transform.translation = new_pos;
}

/// Check if a player-sized AABB at position collides with solid blocks.
fn check_collision(
    pos: Vec3,
    chunk_map: &ChunkMap,
    chunks: &Query<(&ChunkCoord, &ChunkData)>,
) -> bool {
    let half_w = PLAYER_WIDTH / 2.0;
    let min = Vec3::new(pos.x - half_w, pos.y, pos.z - half_w);
    let max = Vec3::new(pos.x + half_w, pos.y + PLAYER_HEIGHT, pos.z + half_w);

    let min_block = IVec3::new(
        min.x.floor() as i32,
        min.y.floor() as i32,
        min.z.floor() as i32,
    );
    let max_block = IVec3::new(
        max.x.floor() as i32,
        max.y.floor() as i32,
        max.z.floor() as i32,
    );

    for by in min_block.y..=max_block.y {
        for bz in min_block.z..=max_block.z {
            for bx in min_block.x..=max_block.x {
                if by < 0 || by >= CHUNK_HEIGHT as i32 {
                    continue;
                }
                let block = get_block(IVec3::new(bx, by, bz), chunk_map, chunks);
                if block.is_solid() {
                    return true;
                }
            }
        }
    }
    false
}

fn get_block(
    world_pos: IVec3,
    chunk_map: &ChunkMap,
    chunks: &Query<(&ChunkCoord, &ChunkData)>,
) -> crate::world::voxel::BlockType {
    use crate::world::voxel::BlockType;

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

/// Auto-grab cursor when entering InGame state.
pub fn grab_cursor_on_enter(
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor) = cursor_q.single_mut() {
        cursor.grab_mode = CursorGrabMode::Confined;
        cursor.visible = false;
    }
}

/// System to grab mouse cursor on left click (Escape handled by pause menu).
pub fn cursor_grab_system(
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Confined;
        cursor.visible = false;
    }
}

/// Emit footstep sounds while the player is moving on ground.
pub fn footstep_sound_system(
    time: Res<Time>,
    mut timer: ResMut<FootstepTimer>,
    player_q: Query<&PlayerController, With<Player>>,
    mut sound_writer: MessageWriter<crate::sound::SoundEvent>,
) {
    let Ok(controller) = player_q.single() else { return };

    let horizontal_speed = Vec3::new(controller.velocity.x, 0.0, controller.velocity.z).length();
    let is_moving = horizontal_speed > 0.5 && controller.on_ground;

    if !is_moving {
        timer.0 = 0.0;
        return;
    }

    timer.0 -= time.delta_secs();
    if timer.0 <= 0.0 {
        let interval = if controller.is_sprinting { 0.3 } else { 0.4 };
        timer.0 = interval;
        sound_writer.write(crate::sound::SoundEvent::FootstepDirt);
    }
}

/// Update camera position based on CameraMode (first-person vs third-person).
pub fn update_camera_position(
    camera_mode: Res<crate::player::character_model::CameraMode>,
    time: Res<Time>,
    mut camera_q: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    let Ok(mut camera_tf) = camera_q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    match *camera_mode {
        crate::player::character_model::CameraMode::FirstPerson => {
            let target = Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0);
            camera_tf.translation = camera_tf.translation.lerp(target, (dt * 12.0).min(1.0));
        }
        crate::player::character_model::CameraMode::ThirdPerson => {
            // Get current pitch from camera rotation
            let (_, pitch, _) = camera_tf.rotation.to_euler(EulerRot::YXZ);

            // Calculate orbit position behind and above the player
            let orbit_offset = Quat::from_rotation_x(pitch) * Vec3::new(0.0, 2.5, -5.0);
            let target = orbit_offset;
            camera_tf.translation = camera_tf.translation.lerp(target, (dt * 12.0).min(1.0));

            // Look at the player's head
            let look_target = Vec3::new(0.0, 1.2, 0.0);
            let look_dir = (look_target - camera_tf.translation).normalize_or_zero();
            if look_dir.length_squared() > 0.001 {
                camera_tf.look_to(look_dir, Vec3::Y);
            }
        }
    }
}
