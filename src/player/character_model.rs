use bevy::prelude::*;

use super::controller::Player;

/// Camera mode: first-person or third-person.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    #[default]
    FirstPerson,
    ThirdPerson,
}

/// Root entity for the character model (child of Player).
#[derive(Component)]
pub struct CharacterModelRoot;

/// Body part type for animation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BodyPart {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

/// Component for animatable limb pivots.
#[derive(Component)]
pub struct CharacterLimb {
    pub part: BodyPart,
}

/// Spawn Ronin's procedural character model as children of the player entity.
pub fn spawn_character_model(
    mut commands: Commands,
    player_q: Query<Entity, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(player_entity) = player_q.single() else {
        return;
    };

    let skin_color = Color::srgb(0.45, 0.3, 0.2);
    let shirt_color = Color::srgb(0.2, 0.6, 0.2); // Green Spider-Man tee
    let shorts_color = Color::srgb(0.6, 0.5, 0.3); // Checkered shorts (tan)
    let hair_color = Color::srgb(0.1, 0.08, 0.05); // Dark dreadlocks

    let skin_mat = materials.add(StandardMaterial { base_color: skin_color, ..default() });
    let shirt_mat = materials.add(StandardMaterial { base_color: shirt_color, ..default() });
    let shorts_mat = materials.add(StandardMaterial { base_color: shorts_color, ..default() });
    let hair_mat = materials.add(StandardMaterial { base_color: hair_color, ..default() });

    // Spawn model root (hidden in first person)
    let model_root = commands.spawn((
        CharacterModelRoot,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Hidden,
    )).id();

    commands.entity(player_entity).add_child(model_root);

    // Head
    let head_mesh = meshes.add(Cuboid::new(0.45, 0.45, 0.45));
    let head = commands.spawn((
        CharacterLimb { part: BodyPart::Head },
        Mesh3d(head_mesh),
        MeshMaterial3d(skin_mat.clone()),
        Transform::from_xyz(0.0, 1.45, 0.0),
    )).id();
    commands.entity(model_root).add_child(head);

    // Dreadlocks (black cubes on top of head)
    let dreads_mesh = meshes.add(Cuboid::new(0.5, 0.2, 0.5));
    let dreads = commands.spawn((
        Mesh3d(dreads_mesh),
        MeshMaterial3d(hair_mat),
        Transform::from_xyz(0.0, 0.3, 0.0),
    )).id();
    commands.entity(head).add_child(dreads);

    // Torso
    let torso_mesh = meshes.add(Cuboid::new(0.5, 0.55, 0.25));
    let torso = commands.spawn((
        CharacterLimb { part: BodyPart::Torso },
        Mesh3d(torso_mesh),
        MeshMaterial3d(shirt_mat),
        Transform::from_xyz(0.0, 0.95, 0.0),
    )).id();
    commands.entity(model_root).add_child(torso);

    // Left Arm (pivot at shoulder)
    let arm_mesh = meshes.add(Cuboid::new(0.18, 0.5, 0.18));
    let left_arm_pivot = commands.spawn((
        CharacterLimb { part: BodyPart::LeftArm },
        Transform::from_xyz(-0.35, 1.15, 0.0),
    )).id();
    let left_arm_mesh = commands.spawn((
        Mesh3d(arm_mesh.clone()),
        MeshMaterial3d(skin_mat.clone()),
        Transform::from_xyz(0.0, -0.25, 0.0),
    )).id();
    commands.entity(left_arm_pivot).add_child(left_arm_mesh);
    commands.entity(model_root).add_child(left_arm_pivot);

    // Right Arm (pivot at shoulder)
    let right_arm_pivot = commands.spawn((
        CharacterLimb { part: BodyPart::RightArm },
        Transform::from_xyz(0.35, 1.15, 0.0),
    )).id();
    let right_arm_mesh = commands.spawn((
        Mesh3d(arm_mesh),
        MeshMaterial3d(skin_mat.clone()),
        Transform::from_xyz(0.0, -0.25, 0.0),
    )).id();
    commands.entity(right_arm_pivot).add_child(right_arm_mesh);
    commands.entity(model_root).add_child(right_arm_pivot);

    // Left Leg (pivot at hip)
    let leg_mesh = meshes.add(Cuboid::new(0.2, 0.5, 0.2));
    let left_leg_pivot = commands.spawn((
        CharacterLimb { part: BodyPart::LeftLeg },
        Transform::from_xyz(-0.13, 0.6, 0.0),
    )).id();
    let left_leg_mesh = commands.spawn((
        Mesh3d(leg_mesh.clone()),
        MeshMaterial3d(shorts_mat.clone()),
        Transform::from_xyz(0.0, -0.25, 0.0),
    )).id();
    commands.entity(left_leg_pivot).add_child(left_leg_mesh);
    commands.entity(model_root).add_child(left_leg_pivot);

    // Right Leg (pivot at hip)
    let right_leg_pivot = commands.spawn((
        CharacterLimb { part: BodyPart::RightLeg },
        Transform::from_xyz(0.13, 0.6, 0.0),
    )).id();
    let right_leg_mesh = commands.spawn((
        Mesh3d(leg_mesh),
        MeshMaterial3d(shorts_mat),
        Transform::from_xyz(0.0, -0.25, 0.0),
    )).id();
    commands.entity(right_leg_pivot).add_child(right_leg_mesh);
    commands.entity(model_root).add_child(right_leg_pivot);
}

/// Toggle camera mode with V key.
pub fn toggle_camera_mode_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_mode: ResMut<CameraMode>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        *camera_mode = match *camera_mode {
            CameraMode::FirstPerson => CameraMode::ThirdPerson,
            CameraMode::ThirdPerson => CameraMode::FirstPerson,
        };
    }
}

/// Show/hide character model based on camera mode.
pub fn update_character_visibility(
    camera_mode: Res<CameraMode>,
    mut model_q: Query<&mut Visibility, With<CharacterModelRoot>>,
) {
    if !camera_mode.is_changed() {
        return;
    }

    let Ok(mut vis) = model_q.single_mut() else { return };
    *vis = match *camera_mode {
        CameraMode::FirstPerson => Visibility::Hidden,
        CameraMode::ThirdPerson => Visibility::Inherited,
    };
}
