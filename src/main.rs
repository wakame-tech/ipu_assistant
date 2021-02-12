use std::env;
use serenity::{prelude::*};
use ipu_assistant::handler::Handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = env::var("DISCORD_ACCESS_TOKEN")
        .expect("Expected a token in environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("client error: {:?}", why);
    }

    Ok(())
}