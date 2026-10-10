use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Serialize)]
pub enum AuthKind {
    None,
    ApiKey { header: &'static str },
    OAuth2(OAuthConfig),
}

#[derive(Serialize)]
pub struct OAuthConfig {
    pub auth_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static [&'static str],
}

pub struct ServiceCtx {
    pub user_id: Uuid,
    pub access_token: Option<String>,
    pub http: reqwest::Client,
}
impl ServiceCtx {
    pub fn token(&self) -> Result<&str, ServiceError> {
        self.access_token.as_deref().ok_or(ServiceError::NotConnected)
    }
}

pub fn parse_params<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, ServiceError> {
    serde_json::from_value(params).map_err(|e| ServiceError::InvalidParams(e.to_string()))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ParamOption {
    pub value: String,
    pub label: String,
}
impl ParamOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into() }
    }

    pub fn same(v: &str) -> Self {
        Self::new(v, v)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ParamSpec {
    pub name: &'static str,

    #[serde(rename = "type")]
    pub param_type: &'static str,

    pub optional: bool,

    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ParamOption>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<u8>,
}
impl Default for ParamSpec {
    fn default() -> Self {
        Self {
            name: "",
            param_type: "",
            optional: false,
            options: None,
            default: None,
            minimum: None,
            maximum: None,
        }
    }
}

#[derive(Serialize)]
pub struct WidgetSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub params_schema: Vec<ParamSpec>
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("unknown widget: {0}")]
    UnknownWidget(String),
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("service not connected")]
    NotConnected,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[async_trait]
pub trait Service: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn label(&self) -> &'static str;

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
            "label": self.label(),
            "auth": self.auth(),
            "widgets": self.widgets().into_iter().map(|w| json!({
                "id": w.id, "name": w.name, "description": w.description, "params": w.params_schema,
            })).collect::<Vec<_>>()
        })
    }

    fn widgets(&self) -> Vec<WidgetSpec>;

    async fn fill_options(
        &self,
        _ctx: &ServiceCtx,
        _widgets: &mut [WidgetSpec]
    ) -> Result<(), ServiceError> {
        Ok(())
    }

    fn auth(&self) -> AuthKind { AuthKind::None }

    async fn identify(
        &self,
        _http: &reqwest::Client,
        _access_token: &str,
    ) -> Result<(String, Value), ServiceError> {
        Err(ServiceError::InvalidParams("provider does not support OAuth".into()))
    }

    async fn fetch_widget(
        &self,
        ctx: &ServiceCtx,
        widget_id: &str,
        params: Value,
    ) -> Result<Value, ServiceError>;
}
