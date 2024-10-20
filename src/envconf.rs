use once_cell::sync::Lazy;


pub const TG_API_HASH: Lazy<String> =
    Lazy::new(|| std::env::var("TG_API_HASH").expect("TG_API_HASH must be set"));
pub const TG_API_ID: Lazy<i32> =
    Lazy::new(|| std::env::var("TG_API_ID").expect("TG_API_ID must be set").parse().unwrap());
pub const ACCOUNT_PHONE: Lazy<String> =
    Lazy::new(|| std::env::var("ACCOUNT_PHONE").expect("ACCOUNT_PHONE must be set"));

pub const HEALTH_CHECK_PERIOD: Lazy<u64> =
    Lazy::new(|| std::env::var("HEALTH_CHECK_PERIOD").expect("HEALTH_CHECK_PERIOD must be set").parse().unwrap());

pub const ALIVE_PATIENCE: Lazy<u64> =
    Lazy::new(|| std::env::var("ALIVE_PATIENCE").expect("ALIVE_PATIENCE must be set").parse().unwrap());

pub const RESTART_PATIENCE: Lazy<u64> =
    Lazy::new(|| std::env::var("RESTART_PATIENCE").expect("RESTART_PATIENCE must be set").parse().unwrap());
