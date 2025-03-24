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
    GenerateWorld,
    LoadSeed,
    Credits,
    Settings,
    Quit,
}

#[derive(Component)]
pub struct MenuComponent;

#[derive(Component)]
pub struct MenuButtonComponent;
pub const SIDEBAR_COLOR: Color = Color::srgb(0.075, 0.075, 0.075);
pub const BUTTON_HOVER_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const TEXT_COLOR: Color = Color::WHITE;
