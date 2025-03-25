use bevy::prelude::{Component, Resource, Timer};

#[derive(Resource)]
pub struct SplashAnimation {
    pub(crate) current_frame: usize,
    pub(crate) phase: SplashPhase,
    pub(crate) timer: Timer,
}

#[derive(Component)]
pub struct SplashScreen;

#[derive(Component)]
pub struct SplashFrame {
    pub(crate) index: usize,
}

#[derive(Component)]
pub struct InfoText;

#[derive(Component)]
pub struct InfoScreen;

// En fait comme je n'ai pas trouvé de décodeur vidéo simple d'utilisation,
// j'ai décidé de faire une animation image par image, puis avec du code :)
#[derive(Debug, PartialEq, Eq)]
pub enum SplashPhase {
    Glitch,     // 30 premières images (1s)
    Hold,       // 1s de pause
    FadeOut,    // 0.25s de fondu
    InfoScreen, // 5s d'affichage
}