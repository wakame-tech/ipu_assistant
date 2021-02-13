use anyhow::{Result};
use regex::Regex;
use serenity::model::channel::Message;
use sqlx::{Pool, Postgres, Row};
use crate::model::{User};
use crate::config::CONFIG;

async fn get_all_users(pool: &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>("select * from users")
        .fetch_all(pool)
        .await?;
    Ok(users)
}

async fn delete_all_users(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query("delete from users")
        .fetch_all(pool)
        .await?;
    Ok(())
}

async fn insert_user(pool: &Pool<Postgres>, id: &String, name: &String) -> Result<(), sqlx::Error> {
    println!("insert user {:?} {:?}", id, name);
    sqlx::query(r#"
        INSERT INTO users (id, name, count) VALUES ($1, $2, 0)
    "#)
    .bind(id)
    .bind(name)
    .fetch_one(pool)
    .await?;

    Ok(())
}

async fn exist_user(pool: &Pool<Postgres>, id: &String) -> Result<bool, sqlx::Error> {
    println!("exist user {:?}", id);
    let exists = sqlx::query(r#"
        select exists (select * from users where id = $1)
    "#)
    .bind(id)
    .map(|row: sqlx::postgres::PgRow| row.get(0))
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

async fn increment_user(pool: &Pool<Postgres>, id: &String, amount: i32) -> Result<(), sqlx::Error> {
    println!("increment user {:?} {:?}", id, amount);
    // let mut tx = pool.begin().await?;
    sqlx::query(r#"
        update users set count = count + $1 where id = $2
    "#)
    .bind(amount)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(())
}

pub async fn process_cmd(msg: &Message) -> Result<Option<String>, sqlx::Error> {
    match &msg.content[..] {
        "!ipu help" => {
            Ok(Some(env!("CARGO_PKG_VERSION").to_string()))
        }
        "!all" => {
            let pool = Pool::<Postgres>::connect(&CONFIG.database_url()).await?;
            let users = get_all_users(&pool).await?;
            let res = users.iter().map(|u| format!("{} {}", u.name, u.count)).collect::<Vec<_>>().join("\n");
            if res.is_empty() {
                Ok(Some("not found".to_string()))
            } else {
                Ok(Some(res))
            }
        }
        "!reset" => {
            let pool = Pool::<Postgres>::connect(&CONFIG.database_url()).await?;
            delete_all_users(&pool).await?;
            pool.close().await;
            Ok(Some("reseted".to_string()))
        }
        _ => {
            let delay_cmd = Regex::new(r"^\+(\d+)").unwrap();

            if let Some(caps) = delay_cmd.captures(&msg.content) {
                let pool = Pool::<Postgres>::connect(&CONFIG.database_url()).await?;
                let amount: i32 = caps.at(1).unwrap().parse().unwrap();
                let id = msg.author.id.to_string();
                let name = msg.author.name.to_string();
                let exists = exist_user(&pool, &id).await?;
                if exists {
                    // present
                    println!("{:?} present", &id);
                    increment_user(&pool, &id, amount).await?;
                    pool.close().await;
                    return Ok(Some("ok".to_string()))
                } else {
                    // absent
                    println!("{:?} absent", &id);
                    insert_user(&pool, &id, &name).await?;
                    increment_user(&pool, &id, amount).await?;
                    pool.close().await;
                    return Ok(Some("ok".to_string()))
                }
            } else {
                Ok(None)
            }
        }
    }
}