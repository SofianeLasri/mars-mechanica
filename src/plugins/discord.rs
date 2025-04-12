use crate::components::discord::{DiscordClient, LastPresenceUpdate, ReconnectionTimer};
use crate::GameState;
use bevy::prelude::{App, Plugin, Res, ResMut, Startup, State, Time, Update};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DiscordPlugin;

impl Plugin for DiscordPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiscordClient>()
            .init_resource::<LastPresenceUpdate>()
            .init_resource::<ReconnectionTimer>()
            .add_systems(Startup, setup_discord)
            .add_systems(Update, (update_discord_presence, handle_reconnection));
    }
}

fn connect_to_discord(app_id: &str, discord_client: &mut Option<DiscordIpcClient>) {
    match DiscordIpcClient::new(app_id) {
        Ok(mut client) => {
            if let Err(err) = client.connect() {
                eprintln!("Failed to connect to Discord: {}", err);
                *discord_client = None;
                return;
            }

            *discord_client = Some(client);
            println!("Connected to Discord successfully!");
        }
        Err(err) => {
            eprintln!("Failed to create Discord client: {}", err);
            *discord_client = None;
        }
    }
}

fn setup_discord(mut discord_client: ResMut<DiscordClient>) {
    const APP_ID: &str = "1360650879262527699";

    connect_to_discord(APP_ID, &mut discord_client.0);
}

fn handle_reconnection(
    mut discord_client: ResMut<DiscordClient>,
    time: Res<Time>,
    mut reconnect_timer: ResMut<ReconnectionTimer>,
) {
    if discord_client.0.is_some() {
        reconnect_timer.attempts = 0;
        return;
    }

    // Ajouter le temps écoulé
    reconnect_timer.time += time.delta_secs();

    // Calculer le délai basé sur les tentatives (backoff exponentiel avec un max de 60 secondes)
    let delay = (2_u32.pow(reconnect_timer.attempts.min(5)) as f32).min(60.0);

    // Si assez de temps s'est écoulé, essayer de se reconnecter
    if reconnect_timer.time >= delay {
        reconnect_timer.time = 0.0;
        reconnect_timer.attempts += 1;

        println!("Attempting to reconnect to Discord (attempt {})", reconnect_timer.attempts);

        // Remplace avec ton propre ID d'application Discord
        const APP_ID: &str = "1234567890123456789";

        connect_to_discord(APP_ID, &mut discord_client.0);
    }
}

fn update_discord_presence(
    mut discord_client: ResMut<DiscordClient>,
    game_state: Res<State<GameState>>,
    time: Res<Time>,
    mut last_update: ResMut<LastPresenceUpdate>,
) {
    // Mettre à jour seulement si nous avons un client
    let Some(client) = &mut discord_client.0 else {
        return;
    };

    // Mettre à jour seulement si l'état du jeu a changé ou toutes les 15 secondes
    // Discord a une limite de taux, donc on ne devrait pas mettre à jour trop fréquemment
    if game_state.get() == &last_update.state
        && time.elapsed().as_secs_f64() - last_update.time < 15.0 {
        return;
    }

    last_update.time = time.elapsed().as_secs_f64();
    last_update.state = *game_state.get();

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let (state_text, details_text) = match game_state.get() {
        GameState::AssetLoading => ("Dans les menus", "Chargement des assets..."),
        GameState::SplashScreen => ("Dans les menus", "Écran d'accueil"),
        GameState::MainMenu => ("Dans les menus", "Menu principal"),
        GameState::SeedInput => ("Dans les menus", "Configuration du monde"),
        GameState::Loading => ("Dans les menus", "Génération du monde"),
        GameState::InGame => ("En jeu", "Explore la planète rouge"),
    };

    let activity = activity::Activity::new()
        .state(state_text)
        .details(details_text)
        .assets(
            activity::Assets::new()
                .large_image("logo")
                .large_text("Mars Mechanica")
                .small_image("sl")
                .small_text("Développé par Sofiane Lasri"),
        )
        .timestamps(activity::Timestamps::new().start(start_time));

    match client.set_activity(activity) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to update Discord presence: {}", err);

            if err.to_string().contains("Connection") {
                discord_client.0 = None;
            }
        }
    }
}