use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub count: i32
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct PeriodicEvent {
    pub id: i32,
    pub event: String,
    pub cron: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub event: String,
    pub date: DateTime<Local>,
}