mod config;
mod extractor;
mod time_converter;
mod times;
mod user_roles;

use std::sync::Arc;

use log::info;
use serenity::Client;

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

    let mut client = Client::builder(config.bot_token())
        .event_handler(user_roles::UserRoleUpdateHandler::new(
            user_role_cache.clone(),
        ))
        .event_handler(time_converter::MessageHandler::new(
            config.clone(),
            user_role_cache.clone(),
        ))
        .await
        .expect("Failed to create discord client");

    if let Err(reason) = client.start().await {
        println!("Failed to start client: {}", reason)
    }
}
