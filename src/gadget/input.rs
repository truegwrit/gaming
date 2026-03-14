use bevy::prelude::*;

use super::gadget::ActiveGadget;
use crate::ui::inventory_screen::InventoryScreenOpen;

/// Cycle gadget form with G (forward) and F (backward).
pub fn gadget_switch_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gadget: ResMut<ActiveGadget>,
    screen_open: Res<InventoryScreenOpen>,
) {
    if screen_open.0 {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyG) {
        gadget.form = gadget.form.next();
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        gadget.form = gadget.form.prev();
    }
}
