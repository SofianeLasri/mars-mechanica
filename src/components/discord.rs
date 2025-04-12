use crate::GameState;
use bevy::prelude::Resource;
use discord_rich_presence::DiscordIpcClient;

#[derive(Resource)]
pub struct DiscordClient(pub(crate) Option<DiscordIpcClient>);

impl Default for DiscordClient {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Resource)]
pub struct LastPresenceUpdate {
    pub(crate) time: f64,
    pub(crate) state: GameState,
}

impl Default for LastPresenceUpdate {
    fn default() -> Self {
        Self {
            time: 0.0,
            state: GameState::AssetLoading,
        }
    }
}

#[derive(Resource)]
pub struct ReconnectionTimer {
    pub(crate) time: f32,
    pub(crate) attempts: u32,
}

impl Default for ReconnectionTimer {
    fn default() -> Self {
        Self {
            time: 0.0,
            attempts: 0,
        }
    }
}