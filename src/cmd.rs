use anyhow::{Result, anyhow};
use regex::Regex;
use serenity::model::channel::Message;
use sqlx::{Pool, Postgres};
use crate::{config::CONFIG, user_repository::*, events_repository::*};
use std::str::FromStr;
use cron::Schedule;
use chrono::{DateTime, Local};
use itertools::Itertools;

/// !help shows commands list
fn help() -> Result<Option<String>, anyhow::Error> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    Ok(Some(
        format!(r#"ipu_assistant ver {}
- `!all: ポイントを確認する`
- `!reset: ポイントをリセットする`
- `+<num>: {{num}}分遅れる`
- `!add_event <event> <cron>`: 定期イベント {{event}} を追加する
"#, &version)
    ))
}

/// !all returns all delays info
async fn all() -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo = UserRepository { pool };
    let users = repo.get_all_users().await?;
    let res = users.iter().map(|u| format!("{} 📃 {}", u.name, u.count)).collect::<Vec<_>>().join("\n");
    if res.is_empty() {
        Ok(Some("not found".to_string()))
    } else {
        Ok(Some(res))
    }
}

/// !reset
async fn reset() -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo = UserRepository { pool };
    repo.delete_all_users().await?;
    Ok(Some("reseted".to_string()))
}

/// +<minutes>
async fn delay(id: &String, name: &String, amount: i32) -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  UserRepository { pool };

    let count = amount / 10;

    let exists = repo.exist_user(&id).await?;
    if !exists {
        repo.insert_user(&id, &name).await?;
    }
    repo.increment_user(&id, count).await?;
    Ok(Some(format!("📃 +{}", count).to_string()))
}

/// returns latest 10 events
async fn events() -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };
    let events = repo.get_all_events().await?;

    let latest_events = events
        .iter()
        .flat_map(|event|
            Schedule::from_str(event.cron.as_str())
                .unwrap()
                .upcoming(Local)
                .take(10)
                .collect::<Vec<DateTime<Local>>>()
        )
        .into_iter()
        .sorted()
        .iter()
        .take(10)
        .map(|d| d.format("%m月%d日(%a) %H時").to_string())
        .map(|s| format!("- {}", s))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(Some(latest_events))
}

/// cron-like -> cron
fn parse_date_pattern(pattern: String) -> Option<String> {
    todo!()
    // let args = pattern.split(" ")
    //     .collect::<Vec<&str>>();
    // if args.len() < 2 {
    //     return None
    // }
    // Some(format!("0 0 {} * * {} *", args[0], args[1]))
}

/// register periodic event
/// !add <event> <cron-pattern>
async fn add_event(event_name: String, cron_pattern: String) -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };

    match Schedule::from_str(&cron_pattern[..]) {
        Ok(schedule) => {
            let schedules = schedule.upcoming(Local)
            .take(5)
            .map(|d| d.format("%m月%d日(%a) %H時").to_string())
            .map(|s| format!("- {}", s))
            .collect::<Vec<String>>()
            .join("\n");

            repo.insert_event(&event_name, &cron_pattern).await?;

            Ok(Some(format!("✨ {} を登録しました\n{}", event_name, schedules)))
        },
        Err(err) => Err(anyhow!(err)),
    }
}

async fn update_event(event_name: String, cron_pattern: String) -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };

    match Schedule::from_str(&cron_pattern[..]) {
        Ok(schedule) => {
            let schedules = schedule.upcoming(Local)
            .take(5)
            .map(|d| d.format("%m月%d日(%a) %H時").to_string())
            .map(|s| format!("- {}", s))
            .collect::<Vec<String>>()
            .join("\n");

            repo.update_event(&event_name, &cron_pattern).await?;

            Ok(Some(format!("✨ {} を更新しました\n{}", event_name, schedules)))
        },
        Err(err) => Err(anyhow!(err)),
    }
}

pub async fn process_cmd(msg: &Message) -> Result<Option<String>, anyhow::Error> {
    match msg.content.as_str() {
        "!help" => help(),
        "!all" => all().await,
        "!reset" => reset().await,
        _ if msg.content.starts_with("!add_event") => {
            let args: Vec<&str> = msg.content.split(" ").collect();
            if args.len() < 3 {
                return Ok(Some("invalid".to_string()))
            }
            let (event, pattern) = (args[1].to_string(), args[2..].join(" "));
            add_event(event, pattern).await
        },
        _ if msg.content.starts_with("!update_event") => {
            let args: Vec<&str> = msg.content.split(" ").collect();
            if args.len() < 3 {
                return Ok(Some("invalid".to_string()))
            }
            let (event, pattern) = (args[1].to_string(), args[2..].join(" "));
            update_event(event, pattern).await
        },
        _ if msg.content.starts_with("!events") => events().await,
        _ if msg.content.starts_with("+") => {
            if let Some(caps) = Regex::new(r"^\+(\d+)").unwrap().captures(&msg.content) {
                let amount: i32 = caps.at(1).unwrap().parse().unwrap();
                let id = msg.author.id.to_string();
                let name = msg.author.name.to_string();
                return delay(&id, &name, amount).await
            }
            Ok(None)
        },
        _ => Ok(None)
    }
}