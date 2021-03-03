#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub count: i32
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Event {
    pub id: i32,
    pub event: String,
    pub cron: String
}