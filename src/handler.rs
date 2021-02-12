use serenity::{
    async_trait, 
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use crate::cmd::{self};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Ok(Some(res)) = cmd::process_cmd(&msg).await {
            println!("res = {:?}", res);
            if let Err(why) = msg.channel_id.say(&ctx.http, res).await {
                println!("Error: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}