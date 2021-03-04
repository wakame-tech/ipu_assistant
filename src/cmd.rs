use anyhow::{Result, anyhow};
use regex::Regex;
use serenity::model::channel::Message;
use sqlx::{Pool, Postgres};
use crate::{config::CONFIG, user_repository::*, events_repository::*};
use std::str::FromStr;
use cron::Schedule;
use chrono::{DateTime, Local};
use itertools::Itertools;

const LIMIT: usize = 10;
const PREVIEW_LIMIT: usize = 5;

/// !help shows commands list
fn help() -> Result<Option<String>, anyhow::Error> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    Ok(Some(
        format!(r#"ipu_assistant ver {}
- `!points ls`: ポイントを確認する
- `!points reset`: ポイントをリセットする
- `+<num>`: {{num}}分遅れる
- `!events ls`: 定期イベントを確認する
- `!events add <event> <cron>`: 定期イベント {{event}} を追加する
- `!events update <event> <cron>`: 定期イベント {{event}} を更新する
- `!events rm <event>`: 定期イベント {{event}} を削除する
"#, &version)
    ))
}

/// !all returns all delays info
async fn ls_points() -> Result<Option<String>, anyhow::Error> {
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
async fn reset_points() -> Result<Option<String>, anyhow::Error> {
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
async fn ls_events() -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };
    let events = repo.get_all_events().await?;

    let mut res: String = "".to_string();

    res += "\n**👀 Events**\n";
    res += events
        .iter()
        .map(|e| format!("- {}: `{}`", e.event, e.cron))
        .collect::<Vec<_>>()
        .join("\n")
        .as_str();
    res += "\n**🔥 Upcoming**\n";
    res += repo.get_latest_events_by_limit(events, LIMIT)
        .map(|event| format!("- {} **{}**", event.date.format("%m/%d(%a) %H~"), event.event))
        .collect::<Vec<_>>()
        .join("\n")
        .as_str();

    Ok(Some(res))
}

/// register periodic event
/// !add <event> <cron-pattern>
async fn add_events(event_name: &String, cron_pattern: &String) -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };

    dbg!("!add_event", event_name, cron_pattern);

    match Schedule::from_str(cron_pattern) {
        Ok(schedule) => {
            let schedules = schedule.upcoming(Local)
            .take(PREVIEW_LIMIT)
            .map(|d| d.format("%m月%d日(%a) %H時").to_string())
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n");

            repo.insert_event(event_name, cron_pattern).await?;

            Ok(Some(format!("✨ {} を登録しました\n{}", event_name, schedules)))
        },
        Err(err) => Err(anyhow!(err)),
    }
}

async fn update_events(event_name: &String, cron_pattern: &String) -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };

    dbg!("!update_event", event_name, cron_pattern);

    match Schedule::from_str(cron_pattern) {
        Ok(schedule) => {
            let schedules = schedule.upcoming(Local)
            .take(PREVIEW_LIMIT)
            .map(|d| d.format("%m月%d日(%a) %H時").to_string())
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n");

            repo.update_event(event_name, cron_pattern).await?;

            Ok(Some(format!("✨ {} を更新しました\n{}", event_name, schedules)))
        },
        Err(err) => Err(anyhow!(err)),
    }
}

async fn delete_events(event_name: &String) -> Result<Option<String>, anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };

    dbg!("!delete_event", event_name);

    match repo.delete_events(event_name).await {
        Ok(_) => Ok(Some(format!("✨ {} を削除しました", event_name))),
        Err(err) => Err(anyhow!(err)),
    }
}

pub async fn process_cmd(msg: &Message) -> Result<Option<String>, anyhow::Error> {
    match msg.content.as_str() {
        "!help" => help(),
        // !points cmds
        _ if msg.content.starts_with("!points") => {
            let args: Vec<&str> = msg.content.split(" ").collect();
            if args.len() < 2 {
                return Ok(Some("invalid args".to_string()))
            }

            match args[1] {
                "ls" => {
                    ls_points().await
                },
                "reset" => {
                    reset_points().await
                },
                _ => Ok(Some("invalid sub cmd".to_string()))
            }
        },
        // !events cmds
         _ if msg.content.starts_with("!events") => {
            let args: Vec<&str> = msg.content.split(" ").collect();
            if args.len() < 2 {
                return Ok(Some("invalid args".to_string()))
            }

            match args[1] {
                "ls" => {
                    ls_events().await
                },
                "add" => {
                    let args: Vec<&str> = msg.content.split(" ").collect();
                    if args.len() < 4 {
                        return Ok(Some("invalid args".to_string()))
                    }
                    let (event, pattern) = (args[2].to_string(), args[3..].join(" "));
                    add_events(&event, &pattern).await
                },
                "update" => {
                    let args: Vec<&str> = msg.content.split(" ").collect();
                    if args.len() < 4 {
                        return Ok(Some("invalid args".to_string()))
                    }
                    let (event, pattern) = (args[2].to_string(), args[3..].join(" "));
                    update_events(&event, &pattern).await
                },
                "rm" => {
                    let args: Vec<&str> = msg.content.split(" ").collect();
                    if args.len() < 3 {
                        return Ok(Some("invalid args".to_string()))
                    }
                    let event = args[2].to_string();
                    delete_events(&event).await
                },
                _ => Ok(Some("invalid cmd".to_string()))
            }
        },
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