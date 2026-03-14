use bevy::prelude::*;

use crate::gadget::gadget::ActiveGadget;

#[derive(Component)]
pub struct GadgetHudText;

/// Spawn gadget form indicator above the hotbar.
pub fn setup_gadget_hud(mut commands: Commands) {
    commands.spawn((
        GadgetHudText,
        Text::new("[Pickaxe]"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(56.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-40.0),
                ..default()
            },
            ..default()
        },
    ));
}

/// Update gadget HUD text and color.
pub fn update_gadget_hud_system(
    gadget: Res<ActiveGadget>,
    mut text_q: Query<(&mut Text, &mut TextColor), With<GadgetHudText>>,
) {
    if !gadget.is_changed() {
        return;
    }

    let Ok((mut text, mut color)) = text_q.single_mut() else {
        return;
    };

    *text = Text::new(format!("[{}]", gadget.form.display_name()));
    *color = TextColor(gadget.form.color());
}
