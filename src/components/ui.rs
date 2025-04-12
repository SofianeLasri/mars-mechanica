use bevy::prelude::{Color, Component};

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct LoadingText;

#[derive(Component)]
pub struct LoadingBar;

#[derive(Component)]
pub struct LoadingProgress;

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

#[derive(Component)]
pub(crate) struct UiCamera;

#[derive(Component)]
pub struct UiSound;

#[derive(Component)]
pub struct SeedInputScreen;

#[derive(Component)]
pub struct SeedInputValue {
    pub value: String,
}

#[derive(Component)]
pub struct SeedInputValueText;

#[derive(Component)]
pub struct SeedIncrementButton;

#[derive(Component)]
pub struct SeedDecrementButton;

#[derive(Component)]
pub struct SeedRandomizeButton;

#[derive(Component)]
pub struct SeedSubmitButton;

#[derive(Component)]
pub struct SeedCancelButton;

pub const SIDEBAR_COLOR: Color = Color::srgb(0.075, 0.075, 0.075);
pub const BUTTON_HOVER_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const TEXT_COLOR: Color = Color::WHITE;

pub const LOADING_BAR_COLOR: Color = Color::srgb(27.0 / 255.0, 27.0 / 255.0, 27.0 / 255.0);
pub const LOADING_BAR_ERROR_COLOR: Color = Color::srgb(0.5, 0.0, 0.0);
pub const LOADING_PROGRESS_COLOR: Color = Color::srgb(215.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0);
