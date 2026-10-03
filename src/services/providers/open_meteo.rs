use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    services::traits::*,
    structs::open_meteo::*
};

pub struct OpenMeteo;

const GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Deserialize)]
struct LocationParams {
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    #[serde(default = "default_units")]
    units: String, // "celsius" | "fahrenheit"
    #[serde(default = "default_days")]
    days: u8, // only used by the forecast widget
}
fn default_units() -> String { "celsius".into() }
fn default_days() -> u8 { 5 }

#[derive(Serialize)]
struct Place {
    name: String,
    country: Option<String>,
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize)]
struct DayForecast {
    date: String,
    weather_code: u8,
    temp_max: f64,
    temp_min: f64,
    precipitation_mm: f64,
}

impl OpenMeteo {
    async fn resolve_place(
        ctx: &ServiceCtx,
        p: &LocationParams,
    ) -> Result<Place, ServiceError> {
        if let (Some(lat), Some(lon)) = (p.latitude, p.longitude) {
            return Ok(Place {
                name: format!("{lat:.2}, {lon:.2}"),
                country: None,
                latitude: lat,
                longitude: lon,
            });
        }

        let city = p.city.as_deref().ok_or_else(|| {
            ServiceError::InvalidParams("provide `city` or `latitude` + `longitude`".into())
        })?;

        let resp: GeocodeResp = ctx
            .http
            .get(GEOCODING_URL)
            .query(&[("name", city), ("count", "1")])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let hit = resp
            .results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| ServiceError::InvalidParams(format!("city not found: {city}")))?;

        Ok(Place {
            name: hit.name,
            country: hit.country,
            latitude: hit.latitude,
            longitude: hit.longitude,
        })
    }

    async fn forecast(
        ctx: &ServiceCtx,
        place: &Place,
        units: &str,
        extra: &[(&str, String)],
    ) -> Result<ForecastResp, ServiceError> {
        let mut query = vec![
            ("latitude", place.latitude.to_string()),
            ("longitude", place.longitude.to_string()),
            ("temperature_unit", units.to_string()),
            ("timezone", "auto".to_string()),
        ];
        query.extend(extra.iter().cloned());

        Ok(ctx
            .http
            .get(FORECAST_URL)
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn current_weather(ctx: &ServiceCtx, p: LocationParams) -> Result<Value, ServiceError> {
        let place = Self::resolve_place(ctx, &p).await?;
        let resp = Self::forecast(
            ctx,
            &place,
            &p.units,
            &[(
                "current",
                "temperature_2m,apparent_temperature,wind_speed_10m,weather_code".into(),
            )],
        )
        .await?;

        let c = resp
            .current
            .ok_or_else(|| ServiceError::Upstream("missing `current` block".into()))?;

        Ok(json!({
            "place": place,
            "temperature": c.temperature_2m,
            "feels_like": c.apparent_temperature,
            "wind_kmh": c.wind_speed_10m,
            "weather_code": c.weather_code,
            "units": p.units,
        }))
    }

    async fn daily_forecast(ctx: &ServiceCtx, p: LocationParams) -> Result<Value, ServiceError> {
        if !(1..=16).contains(&p.days) {
            return Err(ServiceError::InvalidParams("`days` must be 1..=16".into()));
        }
        let place = Self::resolve_place(ctx, &p).await?;
        let resp = Self::forecast(
            ctx,
            &place,
            &p.units,
            &[
                (
                    "daily",
                    "weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum".into(),
                ),
                ("forecast_days", p.days.to_string()),
            ],
        )
        .await?;

        let d = resp.daily
            .ok_or_else(|| ServiceError::Upstream("missing `daily` block".into()))?;

        let days: Vec<DayForecast> = (0..d.time.len())
            .map(|i| DayForecast {
                date: d.time[i].clone(),
                weather_code: d.weather_code[i],
                temp_max: d.temperature_2m_max[i],
                temp_min: d.temperature_2m_min[i],
                precipitation_mm: d.precipitation_sum[i],
            })
            .collect();

        Ok(json!({ "place": place, "units": p.units, "days": days }))
    }
}

#[async_trait]
impl Service for OpenMeteo {
    fn name(&self) -> &'static str {
        "open_meteo"
    }

    fn auth(&self) -> AuthKind {
        AuthKind::None
    }

    fn describe(&self) -> Value {
        json!({
            "name": self.name(),
            "widgets": self.widgets().into_iter().map(|widget| {
                json!({
                    "name": widget.name,
                    "description": widget.description,
                    "params": widget.params_schema.into_iter().map(|param| {
                        json!({
                            "name": param.name,
                            "type": param.param_type,
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        })
    }

    fn describe_catalog(&self) -> Value {
        json!({
            "name": self.name(),
            "label": "Weather",
            "auth": self.auth(),
            "widgets": self.widgets().into_iter().map(|w| json!({
                "id": w.id,
                "name": w.name,
                "description": w.description,
                "params": w.params_schema,
            })).collect::<Vec<_>>()
        })
    }

    fn widgets(&self) -> Vec<WidgetSpec> {
        let location_schema: Vec<ParamSpec> = vec![
            ParamSpec { name: "city", param_type: "string", ..Default::default() },
            ParamSpec { name: "units", param_type: "string", options: Some(vec!["celsius", "fahrenheit"]),
                default: Some(json!("celsius")), ..Default::default() },
            ParamSpec { name: "latitude", param_type: "number", optional: true, ..Default::default()},
            ParamSpec { name: "longitude", param_type: "number", optional: true, ..Default::default() },
        ];

        let mut forecast_schema = location_schema.clone();
        forecast_schema.push(
            ParamSpec { name: "days", param_type: "integer", optional: false,
                options: None, minimum: Some(1), maximum: Some(16), default: Some(json!(5)) }
        );

        vec![
            WidgetSpec {
                id: "current",
                name: "Current weather",
                description: "Temperature, feels-like and wind for a location.",
                params_schema: location_schema
            },
            WidgetSpec {
                id: "forecast",
                name: "Daily forecast",
                description: "Min/max temperature and rain for the next days.",
                params_schema: forecast_schema
            },
        ]
    }

    async fn fetch_widget(
        &self,
        ctx: &ServiceCtx,
        widget_id: &str,
        params: Value,
    ) -> Result<Value, ServiceError> {
        let p: LocationParams = serde_json::from_value(params)
            .map_err(|e| ServiceError::InvalidParams(e.to_string()))?;

        match widget_id {
            "current" => Self::current_weather(ctx, p).await,
            "forecast" => Self::daily_forecast(ctx, p).await,
            other => Err(ServiceError::UnknownWidget(other.into())),
        }
    }
}
