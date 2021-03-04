use cron::Schedule;
use sqlx::{Pool, Postgres};
use crate::model::{Event, PeriodicEvent};
use anyhow::{Result};
use std::str::FromStr;
use chrono::{Date, DateTime, Local};
use itertools::Itertools;

#[derive(Clone)]
pub struct EventRepository {
    pub pool: Pool<Postgres>
}

impl EventRepository {
    pub async fn get_all_events(&self) -> Result<Vec<PeriodicEvent>, sqlx::Error> {
        let events = sqlx::query_as::<_, PeriodicEvent>(
            "SELECT * FROM events"
        )
            .fetch_all(&self.pool)
            .await?;
        Ok(events)
    }

    pub async fn delete_events(&self, event_name: &String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM events WHERE event = $1")
            .bind(event_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_all_events(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM events")
            .fetch_all(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_event(&self, event_name: &String, cron: &String) -> Result<PeriodicEvent, sqlx::Error> {
        dbg!("insert_event", event_name, cron);
        let event = sqlx::query_as::<_, PeriodicEvent>(r#"
                INSERT INTO events (event, cron) VALUES ($1, $2)
            "#)
            .bind(event_name)
            .bind(cron)
            .fetch_one(&self.pool)
            .await?;

        Ok(event)
    }

    pub async fn update_event(&self, event_name: &String, cron: &String) -> Result<(), sqlx::Error> {
        dbg!("update_event", event_name, cron);
        sqlx::query(r#"
                UPDATE events SET cron = $2 WHERE event = $1
            "#)
            .bind(event_name)
            .bind(cron)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub fn get_latest_events_by_limit(&self, periodic_events: Vec<PeriodicEvent>, limit: usize) -> impl Iterator<Item = Event> {
         periodic_events
            .iter()
            .flat_map(|event|
                Schedule::from_str(event.cron.as_str())
                    .unwrap()
                    .upcoming(Local)
                    .take(limit)
                    .map(|date|Event { event: event.event.clone(), date })
                    .collect::<Vec<_>>()
            )
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
        .into_iter()
        .take(limit)
        .into_iter()
    }

    pub fn get_latest_events_until(&self, periodic_events: Vec<PeriodicEvent>, to: &DateTime<Local>) -> impl Iterator<Item = Event> {
         periodic_events
            .iter()
            .flat_map(|event|
                Schedule::from_str(event.cron.as_str())
                    .unwrap()
                    .upcoming(Local)
                    .take_while(|d| d <= to )
                    .map(|date| Event { event: event.event.clone(), date })
                    .collect::<Vec<_>>()
            )
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
        .into_iter()
    }
}