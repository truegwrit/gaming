use bevy::prelude::*;

/// Temporary knockback impulse applied to an entity.
#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec3,
    pub decay: f32,
}

/// Apply knockback velocity and decay it over time.
pub fn apply_knockback_system(
    time: Res<Time>,
    mut commands: Commands,
    mut knockback_q: Query<(Entity, &mut Transform, &mut Knockback)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut knockback) in knockback_q.iter_mut() {
        transform.translation += knockback.velocity * dt;
        let decay_factor = (1.0 - knockback.decay * dt).max(0.0);
        knockback.velocity *= decay_factor;

        if knockback.velocity.length_squared() < 0.01 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}
