mod times;

use std::env;

use serenity::Client;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Warn)
        .filter_module("server_manager", log::LevelFilter::Debug)
        .init();

    let token = env::var("BOT_TOKEN").expect("Discord bot token environment variable not set");

    let mut client = Client::builder(token)
        .event_handler(times::Handler)
        .await
        .expect("Failed to create discord client");

    if let Err(reason) = client.start().await {
        println!("Failed to start client: {}", reason)
    }
}
