use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};

use crate::{
    constants::SOUNDCLOUD_API,
    services::traits::*,
    structs::soundcloud::*,
};

#[derive(Deserialize)]
struct PlaylistParams {
    playlist_id: String,
}

pub struct SoundCloud;

fn embed_url(kind: &str, id: u64) -> String {
    let api_url = format!("{SOUNDCLOUD_API}/{kind}/{id}");
    reqwest::Url::parse_with_params(
        "https://w.soundcloud.com/player/",
        &[("url", api_url.as_str()), ("auto_play", "false"), ("visual", "false")],
    )
    .map(|u| u.to_string())
    .unwrap_or_default()
}

struct Sc<'a> {
    ctx: &'a ServiceCtx,
}

impl<'a> Sc<'a> {
    async fn api_get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ServiceError> {
        SoundCloud::get(
            &self.ctx.http,
            self.ctx.token()?,
            &format!("{SOUNDCLOUD_API}{path}"),
        )
        .await
    }

    async fn playlists(&self) -> Vec<ParamOption> {
        match self
            .api_get::<ScPage<ScPlaylist>>("/me/playlists?limit=50&show_tracks=false")
            .await
        {
            Ok(page) => page
                .into_items()
                .into_iter()
                .map(|p| ParamOption::new(p.id.to_string(), p.title))
                .collect(),
            Err(e) => {
                tracing::warn!("soundcloud: failed to list playlists: {e}");
                Vec::new()
            }
        }
    }

    async fn playlist_widget(&self, params: Value) -> Result<Value, ServiceError> {
        let p: PlaylistParams = parse_params(params)?;

        if p.playlist_id.is_empty() || !p.playlist_id.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ServiceError::InvalidParams("`playlist_id` must be numeric".into()));
        }

        let pl: ScPlaylistDetail = self
            .api_get(&format!("/playlists/{}?show_tracks=true", p.playlist_id))
            .await?;

        let tracks: Vec<Value> = pl
            .tracks
            .unwrap_or_default()
            .into_iter()
            .map(|t| {
                json!({
                    "id": t.id,
                    "title": t.title,
                    "artist": t.user.and_then(|u| u.username),
                    "duration_ms": t.duration,
                    "artwork_url": t.artwork_url,
                    "permalink_url": t.permalink_url,
                    "access": t.access,
                    "embed_url": embed_url("tracks", t.id),
                })
            })
            .collect();

        Ok(json!({
            "playlist": {
                "id": pl.id,
                "title": pl.title,
                "artwork_url": pl.artwork_url,
                "permalink_url": pl.permalink_url,
                "embed_url": embed_url("playlists", pl.id),
            },
            "tracks": tracks,
        }))
    }
}

impl SoundCloud {
    const CONFIG: OAuthConfig = OAuthConfig {
        auth_url: "https://secure.soundcloud.com/authorize",
        token_url: "https://secure.soundcloud.com/oauth/token",
        scopes: &[],
    };

    async fn get<T: DeserializeOwned>(
        http: &reqwest::Client,
        token: &str,
        url: &str,
    ) -> Result<T, ServiceError> {
        let resp = http
            .get(url)
            .header("Authorization", format!("OAuth {token}"))
            .header("accept", "application/json; charset=utf-8")
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ServiceError::NotConnected);
        }
        Ok(resp.error_for_status()?.json().await?)
    }
}

#[async_trait]
impl Service for SoundCloud {
    fn name(&self) -> &'static str { "soundcloud" }

    fn label(&self) -> &'static str { "SoundCloud" }

    fn auth(&self) -> AuthKind { AuthKind::OAuth2(Self::CONFIG) }

    fn widgets(&self) -> Vec<WidgetSpec> {
        vec![WidgetSpec {
            id: "playlist",
            name: "Playlist player",
            description: "Pick one of your SoundCloud playlists and play it in the dashboard.",
            params_schema: vec![ParamSpec {
                name: "playlist_id",
                param_type: "string",
                ..Default::default()
            }],
        }]
    }

    async fn identify(&self, http: &reqwest::Client, token: &str)
        -> Result<(String, Value), ServiceError> {
        let me: ScMe = Self::get(http, token, &format!("{SOUNDCLOUD_API}/me")).await?;
        Ok((
            me.id.to_string(),
            json!({
                "username": me.username,
                "avatar_url": me.avatar_url,
                "permalink_url": me.permalink_url,
            }),
        ))
    }

    async fn fill_options(&self, ctx: &ServiceCtx, widgets: &mut [WidgetSpec])
        -> Result<(), ServiceError> {
        let playlists = Sc { ctx }.playlists().await;
        for w in widgets.iter_mut().filter(|w| w.id == "playlist") {
            for p in w.params_schema.iter_mut().filter(|p| p.name == "playlist_id") {
                p.options = Some(playlists.clone());
            }
        }
        Ok(())
    }

    async fn fetch_widget(&self, ctx: &ServiceCtx, widget_id: &str, params: Value)
        -> Result<Value, ServiceError> {
        let sc = Sc { ctx };
        match widget_id {
            "playlist" => sc.playlist_widget(params).await,
            other => Err(ServiceError::UnknownWidget(other.into())),
        }
    }
}
