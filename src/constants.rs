use std::time::{Duration};

pub const WEB_ROOT: &str = "./web/dist/web-angular/browser";

pub const REFRESH_TOKEN_LIFETIME_DAYS: i64 = 30;
pub const JWT_LIFETIME_DAYS: i64 = 1;

pub const STATE_TTL: Duration = Duration::from_secs(600);

pub const COOKIE_PATH: &str = "/";

pub const FRONT_URL: &str = "http://localhost:8080";
pub const API_URL: &str = "http://localhost:8080/api"; // Maybe run them on two diffent ports

pub const SOUNDCLOUD_API: &str = "https://api.soundcloud.com";
