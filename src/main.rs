mod composite_event_handler;
mod config;
mod extractor;
mod time_converter;
mod user_roles;

use std::sync::Arc;

use composite_event_handler::CompositeEventHandler;
use log::info;
use serenity::{client::bridge::gateway::GatewayIntents, Client};

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Warn)
        .filter_module("discord_server_manager", log::LevelFilter::Debug)
        .init();

    let config = Arc::new(config::Config::load());
    info!("Loaded Configuration: {:?}", &config);

    let user_role_cache = Arc::new(user_roles::UserRoleCache::new());
    info!("Created user role cache");

    let composite_event_handler = CompositeEventHandler::new()
        .event_handler(time_converter::MessageHandler::new(
            config.clone(),
            user_role_cache.clone(),
        ))
        .event_handler(user_roles::UserRoleUpdateHandler::new(
            user_role_cache.clone(),
        ));

    let mut client = Client::builder(config.bot_token())
        .intents(
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILD_MESSAGE_REACTIONS
                | GatewayIntents::GUILD_MEMBERS,
        )
        .event_handler(composite_event_handler)
        .await
        .expect("Failed to create discord client.");

    if let Err(reason) = client.start().await {
        println!("Failed to start client: {:?}", reason)
    }
}
