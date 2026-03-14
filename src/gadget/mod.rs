pub mod gadget;
pub mod input;

use bevy::prelude::*;

use gadget::ActiveGadget;

pub struct GadgetPlugin;

impl Plugin for GadgetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveGadget>()
            .add_systems(Update, input::gadget_switch_system);
    }
}
