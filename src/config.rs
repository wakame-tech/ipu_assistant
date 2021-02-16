use once_cell::sync::Lazy;
pub struct Config {
    database_url: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config {
    database_url: std::env::var("DATABASE_URL").unwrap(),
});