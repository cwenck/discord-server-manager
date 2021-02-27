mod config;
mod extractor;
mod time_converter;
mod times;
mod user_roles;

use log::info;
use serenity::Client;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Warn)
        .filter_module("discord_server_manager", log::LevelFilter::Debug)
        .init();

    let config = config::Config::load();
    info!("Loaded Configuration: {:?}", &config);

    let mut client = Client::builder(config.bot_token())
        .event_handler(times::Handler::new(&config))
        .await
        .expect("Failed to create discord client");

    if let Err(reason) = client.start().await {
        println!("Failed to start client: {}", reason)
    }
}
