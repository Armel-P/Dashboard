use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use uuid::Uuid;

pub enum AuthKind {
    None,
    ApiKey { header: &'static str },
    OAuth2(OAuthConfig),
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct ParamSpec {
    pub name: &'static str,

    #[serde(rename = "type")]
    pub param_type: &'static str,

    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<&'static str>>,

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
    pub params_schema: Vec<ParamSpec>,
    pub refresh_secs: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("unknown widget: {0}")]
    UnknownWidget(String),
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("service not connected")]
    NotConnected,
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[async_trait]
pub trait Service: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn auth(&self) -> AuthKind;
    fn describe(&self) -> Value;
    fn widgets(&self) -> Vec<WidgetSpec>;

    async fn fetch_widget(
        &self,
        ctx: &ServiceCtx,
        widget_id: &str,
        params: Value,
    ) -> Result<Value, ServiceError>;
}
