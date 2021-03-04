use serenity::{prelude::*};
use ipu_assistant::{
    config::CONFIG,
    handler::Handler, 
    scheduler::scheduler
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::builder(&CONFIG.discord_api_token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    scheduler();

    // bot
    if let Err(why) = client.start().await {
        println!("client error: {:?}", why);
    }

    Ok(())
}