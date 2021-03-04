use crate::{config::{CONFIG}, events_repository::{EventRepository}};
use chrono::{DateTime, Duration, Local};
use job_scheduler::{JobScheduler, Job};
use sqlx::{Pool, Postgres};
use tokio::runtime::{Runtime};
use std::{collections::HashMap};

/// check interval = 5s
const INTERVAL: &str = "0 * * * * *";
const DELTA_SECONDS: i64 = 120;

/// notify before seconds
const BEFORE_SECONDS: i64 = 60 * 10;

/// push message via incoming webhook
async fn push_message(content: &String) -> Result<(), anyhow::Error> {
    let mut body = HashMap::new();
    body.insert("content", content);
    dbg!(&body);
    let client = reqwest::Client::new();
    let res = client.post(&CONFIG.incoming_webhook_url)
        .json(&body)
        .send()
        .await?;
    let json = res.text().await?;
    dbg!(json);
    Ok(())
}

/// b in? a +- delta
fn is_near(a: DateTime<Local>, b: DateTime<Local>, delta: i64) -> bool {
    let early = a + Duration::seconds(-delta);
    let late = a + Duration::seconds(delta);
    early <= b && b <= late
}

/// check events date
pub async fn notify_events() -> Result<(), anyhow::Error> {
    let pool = Pool::<Postgres>::connect(&CONFIG.database_url.to_string()).await?;
    let repo =  EventRepository { pool };
    let periodic_events = repo.get_all_events().await?;
    let to = Local::now() + Duration::seconds(BEFORE_SECONDS);
    let nearly_events = repo.get_latest_events_until(periodic_events, &to)
        .filter(|event| is_near(event.date, Local::now(), DELTA_SECONDS));
    for event in nearly_events {
        let message = format!("✔ イベント **{}**({}~) の時間が近づいています", event.event, event.date.format("%m/%d(%a) %H"));
        push_message(&message).await?
    }

    Ok(())
}

/// polling events every [INTERVAL]
pub fn scheduler() {
    // scheduler
    std::thread::spawn(move || {
        let mut rt = Runtime::new().unwrap();
        let mut sched = JobScheduler::new();
        sched.add(Job::new(INTERVAL.parse().unwrap(), || {
            rt.block_on( async {
                notify_events().await;
            });
        }));
        loop {
            sched.tick();
            std::thread::sleep(sched.time_till_next_job());
        }
    });
}