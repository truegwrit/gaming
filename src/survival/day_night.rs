use bevy::prelude::*;
use std::f32::consts::PI;

/// Resource tracking world time.
#[derive(Resource)]
pub struct DayCycle {
    /// 0.0..1.0 normalized time. 0.0=dawn, 0.25=noon, 0.5=dusk, 0.75=midnight
    pub time: f32,
    /// Seconds for one full cycle (default 1200 = 20 minutes)
    pub cycle_duration: f32,
    /// Current day number
    pub day: u32,
}

impl Default for DayCycle {
    fn default() -> Self {
        Self {
            time: 0.1, // Start shortly after dawn
            cycle_duration: 1200.0,
            day: 1,
        }
    }
}

impl DayCycle {
    pub fn time_of_day_name(&self) -> &'static str {
        match self.time {
            t if t < 0.05 || t > 0.95 => "Dawn",
            t if t < 0.20 => "Morning",
            t if t < 0.30 => "Noon",
            t if t < 0.45 => "Afternoon",
            t if t < 0.55 => "Dusk",
            t if t < 0.70 => "Evening",
            _ => "Night",
        }
    }
}

/// Marker component for the sun DirectionalLight.
#[derive(Component)]
pub struct Sun;

/// Spawn the sun and lighting.
pub fn setup_sun(mut commands: Commands) {
    commands.spawn((
        Sun,
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.6, 0.7, 1.0),
        brightness: 500.0,
        affects_lightmapped_meshes: true,
    });

    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0)));
}

/// Advance the day/night cycle time.
pub fn advance_time_system(time: Res<Time>, mut cycle: ResMut<DayCycle>) {
    cycle.time += time.delta_secs() / cycle.cycle_duration;
    if cycle.time >= 1.0 {
        cycle.time -= 1.0;
        cycle.day += 1;
    }
}

/// Rotate the sun based on time of day and adjust illuminance.
pub fn update_sun_system(
    cycle: Res<DayCycle>,
    mut sun_q: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
) {
    let Ok((mut transform, mut light)) = sun_q.single_mut() else {
        return;
    };

    // Sun elevation: peaks at noon (time=0.25), below horizon at midnight (time=0.75)
    let angle = cycle.time * 2.0 * PI;
    let elevation = (angle - PI / 2.0).sin(); // -1 at dawn, 1 at noon-ish, -1 at dusk

    // Sun rotation
    let pitch = -elevation.acos() + PI / 2.0;
    let azimuth = 0.4; // Fixed east-west orientation
    *transform = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, pitch, azimuth, 0.0));

    // Illuminance based on sun elevation
    if elevation > 0.0 {
        light.illuminance = 10000.0 * elevation.powf(0.5);
    } else {
        light.illuminance = 0.0;
    }
}

/// Update ambient light color and brightness based on time of day.
pub fn update_ambient_light_system(
    cycle: Res<DayCycle>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let t = cycle.time;

    // Brightness: high during day, low at night
    let brightness = if t < 0.05 || t > 0.95 {
        // Dawn/pre-dawn
        lerp(100.0, 400.0, smoothstep(0.95, 0.05, if t > 0.5 { t - 1.0 } else { t }))
    } else if t < 0.45 {
        // Day
        500.0
    } else if t < 0.55 {
        // Dusk
        lerp(500.0, 50.0, smoothstep(0.45, 0.55, t))
    } else {
        // Night
        50.0
    };

    // Color: warm at dawn/dusk, blue-white during day, dark blue at night
    let color = if t < 0.05 || t > 0.95 {
        Color::srgb(0.9, 0.6, 0.4) // Warm dawn
    } else if t < 0.45 {
        Color::srgb(0.6, 0.7, 1.0) // Daylight blue-white
    } else if t < 0.55 {
        Color::srgb(0.9, 0.5, 0.3) // Warm dusk
    } else {
        Color::srgb(0.1, 0.1, 0.3) // Night blue
    };

    ambient.brightness = brightness;
    ambient.color = color;
}

/// Update sky color based on time of day.
pub fn update_sky_color_system(
    cycle: Res<DayCycle>,
    mut clear_color: ResMut<ClearColor>,
) {
    let t = cycle.time;

    let sky = if t < 0.05 || t > 0.95 {
        Color::srgb(0.8, 0.5, 0.3) // Dawn orange
    } else if t < 0.20 {
        let f = smoothstep(0.05, 0.20, t);
        lerp_color(Color::srgb(0.8, 0.5, 0.3), Color::srgb(0.5, 0.7, 1.0), f)
    } else if t < 0.45 {
        Color::srgb(0.5, 0.7, 1.0) // Day blue
    } else if t < 0.55 {
        let f = smoothstep(0.45, 0.55, t);
        lerp_color(Color::srgb(0.5, 0.7, 1.0), Color::srgb(0.8, 0.4, 0.2), f)
    } else if t < 0.65 {
        let f = smoothstep(0.55, 0.65, t);
        lerp_color(Color::srgb(0.8, 0.4, 0.2), Color::srgb(0.05, 0.05, 0.15), f)
    } else {
        Color::srgb(0.05, 0.05, 0.15) // Night dark
    };

    clear_color.0 = sky;
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgb(
        lerp(a.red, b.red, t),
        lerp(a.green, b.green, t),
        lerp(a.blue, b.blue, t),
    )
}
