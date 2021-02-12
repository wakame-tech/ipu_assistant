#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub count: i32
}