use bevy::prelude::{Color, Component};

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct LoadingText;

#[derive(Component)]
pub struct MenuButton {
    pub(crate) action: ButtonAction,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonAction {
    Play,
    Quit,
}

// Constantes de style
pub const BUTTON_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BUTTON_HOVER: Color = Color::srgb(0.25, 0.25, 0.25);
pub const BUTTON_PRESS: Color = Color::srgb(0.35, 0.75, 0.35);
pub const TEXT_COLOR: Color = Color::WHITE;
