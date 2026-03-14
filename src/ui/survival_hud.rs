use bevy::prelude::*;

use crate::player::controller::Player;
use crate::survival::health::Health;
use crate::survival::hunger::Hunger;

#[derive(Component)]
pub struct HealthBarSegment(pub usize);

#[derive(Component)]
pub struct HungerBarSegment(pub usize);

/// Spawn health and hunger bar UI elements.
pub fn setup_survival_hud(mut commands: Commands) {
    // Health bar container (bottom-left area)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(25.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(2.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 0..10 {
                // Heart background (dark red)
                parent
                    .spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.0, 0.0, 0.8)),
                    ))
                    .with_children(|heart| {
                        // Heart fill (bright red)
                        heart.spawn((
                            HealthBarSegment(i),
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.9, 0.1, 0.1, 1.0)),
                        ));
                    });
            }
        });

    // Hunger bar container (bottom-right area)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                right: Val::Percent(25.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(2.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 0..10 {
                // Hunger background (dark brown)
                parent
                    .spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.1, 0.0, 0.8)),
                    ))
                    .with_children(|hunger| {
                        // Hunger fill (tan/brown)
                        hunger.spawn((
                            HungerBarSegment(i),
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.7, 0.5, 0.2, 1.0)),
                        ));
                    });
            }
        });
}

/// Update health bar segments based on player health.
pub fn update_health_bar_system(
    player_q: Query<&Health, With<Player>>,
    mut segments: Query<(&HealthBarSegment, &mut BackgroundColor)>,
) {
    let Ok(health) = player_q.single() else {
        return;
    };

    for (segment, mut color) in segments.iter_mut() {
        let threshold = (segment.0 as f32 + 1.0) * 2.0; // Each segment = 2 HP
        let half_threshold = segment.0 as f32 * 2.0 + 1.0;

        if health.current >= threshold {
            // Full heart
            *color = BackgroundColor(Color::srgba(0.9, 0.1, 0.1, 1.0));
        } else if health.current >= half_threshold {
            // Half heart
            *color = BackgroundColor(Color::srgba(0.9, 0.1, 0.1, 0.5));
        } else {
            // Empty heart
            *color = BackgroundColor(Color::srgba(0.9, 0.1, 0.1, 0.1));
        }
    }
}

/// Update hunger bar segments based on player hunger.
pub fn update_hunger_bar_system(
    player_q: Query<&Hunger, With<Player>>,
    mut segments: Query<(&HungerBarSegment, &mut BackgroundColor)>,
) {
    let Ok(hunger) = player_q.single() else {
        return;
    };

    for (segment, mut color) in segments.iter_mut() {
        let threshold = (segment.0 as f32 + 1.0) * 2.0;
        let half_threshold = segment.0 as f32 * 2.0 + 1.0;

        if hunger.current >= threshold {
            *color = BackgroundColor(Color::srgba(0.7, 0.5, 0.2, 1.0));
        } else if hunger.current >= half_threshold {
            *color = BackgroundColor(Color::srgba(0.7, 0.5, 0.2, 0.5));
        } else {
            *color = BackgroundColor(Color::srgba(0.7, 0.5, 0.2, 0.1));
        }
    }
}
