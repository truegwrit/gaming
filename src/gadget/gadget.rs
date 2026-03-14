use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GadgetForm {
    #[default]
    Pickaxe,
    Sword,
    Axe,
    Shovel,
}

impl GadgetForm {
    pub fn next(self) -> Self {
        match self {
            Self::Pickaxe => Self::Sword,
            Self::Sword => Self::Axe,
            Self::Axe => Self::Shovel,
            Self::Shovel => Self::Pickaxe,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Pickaxe => Self::Shovel,
            Self::Sword => Self::Pickaxe,
            Self::Axe => Self::Sword,
            Self::Shovel => Self::Axe,
        }
    }

    pub fn attack_damage(self) -> f32 {
        match self {
            Self::Sword => 7.0,
            Self::Axe => 5.0,
            Self::Pickaxe => 3.0,
            Self::Shovel => 2.0,
        }
    }

    pub fn attack_reach(self) -> f32 {
        match self {
            Self::Sword => 4.0,
            _ => 3.0,
        }
    }

    pub fn attack_cooldown(self) -> f32 {
        match self {
            Self::Sword => 0.5,
            _ => 1.0,
        }
    }

    pub fn mining_speed_multiplier(self) -> f32 {
        match self {
            Self::Pickaxe => 2.0,
            Self::Axe => 1.5,
            Self::Shovel => 1.5,
            Self::Sword => 0.5,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Pickaxe => "Pickaxe",
            Self::Sword => "Sword",
            Self::Axe => "Axe",
            Self::Shovel => "Shovel",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Pickaxe => Color::srgb(0.6, 0.6, 0.7),
            Self::Sword => Color::srgb(0.9, 0.2, 0.2),
            Self::Axe => Color::srgb(0.7, 0.5, 0.2),
            Self::Shovel => Color::srgb(0.5, 0.4, 0.3),
        }
    }

    /// Returns true if this form can mine blocks.
    pub fn can_mine(self) -> bool {
        self != Self::Sword
    }
}

#[derive(Resource, Default)]
pub struct ActiveGadget {
    pub form: GadgetForm,
}
