use once_cell::sync::Lazy;
pub struct Config {
    pub database_url: String,
    pub discord_api_token: String,
    pub incoming_webhook_url: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config {
    database_url: std::env::var("DATABASE_URL").unwrap(),
    discord_api_token: std::env::var("DISCORD_ACCESS_TOKEN").unwrap(),
    incoming_webhook_url: std::env::var("INCOMING_WEBHOOK_URL").unwrap(),
});