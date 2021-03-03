use sqlx::{Pool, Postgres};
use crate::model::{User, Event};

#[derive(Clone)]
pub struct EventRepository {
    pub pool: Pool<Postgres>
}

impl EventRepository {
    pub async fn get_all_events(&self) -> Result<Vec<Event>, sqlx::Error> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT * FROM events"
        )
            .fetch_all(&self.pool)
            .await?;
        Ok(events)
    }

    pub async fn delete_all_events(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM events")
            .fetch_all(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_event(&self, event_name: &String, cron: &String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(r#"
                INSERT INTO events (event, cron) VALUES ($1, $2)
            "#)
            .bind(event_name)
            .bind(cron)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn exist_user(&self, id: &String) -> Result<bool, sqlx::Error> {
        let users = sqlx::query_as::<_, User>(r#"
            SELECT * FROM users WHERE id = $1
            "#)
            .bind(id)
            .fetch_all(&self.pool)
            .await?;

        Ok(users.len() != 0)
    }

    pub async fn update_event(&self, event_name: &String, cron: &String) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
                UPDATE events SET cron = $2 WHERE event = $1
            "#)
            .bind(event_name)
            .bind(cron)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}