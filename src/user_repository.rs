use sqlx::{Pool, Postgres};
use crate::model::{User};
#[derive(Clone)]
pub struct UserRepository {
    pub pool: Pool<Postgres>
}

impl UserRepository {
    pub async fn get_all_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users"
        )
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    pub async fn delete_all_users(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users")
            .fetch_all(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_user(&self, id: &String, name: &String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(r#"
                INSERT INTO users (id, name, count) VALUES ($1, $2, 0)
            "#)
            .bind(id)
            .bind(name)
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

    pub async fn increment_user(&self, id: &String, amount: i32) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
                UPDATE users SET count = count + $1 WHERE id = $2
            "#)
            .bind(amount)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}