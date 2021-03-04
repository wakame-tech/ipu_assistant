use serenity::{async_trait, model::{channel::Message, gateway::Ready, id::GuildId, prelude::VoiceState}, prelude::*};
use crate::cmd::{self};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match cmd::process_cmd(&msg).await {
            Ok(Some(res)) => {
                println!("res = {:?}", res);
                if let Err(err) = msg.channel_id.say(&ctx.http, res).await {
                    println!("Error: {:?}", err);
                }
            }
            Err(err) => {
                println!("{}", err);
                if let Err(err) = msg.channel_id.say(&ctx.http, err.to_string()).await {
                    println!("Error: {:?}", err);
                }
            }
            _ => {}
        }
    }

    async fn voice_state_update(&self, _ctx: Context, _: Option<GuildId>, state: VoiceState) {
        match state.channel_id {
            Some(_) => dbg!("join {}", state.user_id),
            None => dbg!("leave {}", state.user_id),
        };
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}