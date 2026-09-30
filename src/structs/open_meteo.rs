use serde::{Deserialize};

#[derive(Deserialize)]
pub struct GeocodeResp {
    pub results: Option<Vec<GeoResult>>,
}
#[derive(Deserialize)]
pub struct GeoResult {
    pub name: String,
    pub country: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize)]
pub struct ForecastResp {
    pub current: Option<Current>,
    pub daily: Option<Daily>,
}
#[derive(Deserialize)]
pub struct Current {
    pub temperature_2m: f64,
    pub apparent_temperature: f64,
    pub wind_speed_10m: f64,
    pub weather_code: u8,
}
#[derive(Deserialize)]
pub struct Daily {
    pub time: Vec<String>,
    pub weather_code: Vec<u8>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
}
