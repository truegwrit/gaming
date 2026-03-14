use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseScroll;

#[derive(Resource)]
pub struct HotbarSelection {
    pub selected: usize,
}

impl Default for HotbarSelection {
    fn default() -> Self {
        Self { selected: 0 }
    }
}

/// System to handle hotbar selection via number keys and scroll wheel.
pub fn hotbar_selection_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut hotbar: ResMut<HotbarSelection>,
) {
    // Number keys 1-9
    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    for (i, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            hotbar.selected = i;
            return;
        }
    }

    // Scroll wheel
    let scroll = mouse_scroll.delta.y;
    if scroll > 0.5 {
        hotbar.selected = (hotbar.selected + 8) % 9; // scroll up = previous
    } else if scroll < -0.5 {
        hotbar.selected = (hotbar.selected + 1) % 9; // scroll down = next
    }
}
