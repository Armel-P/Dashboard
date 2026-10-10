use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ScMe {
    pub id: u64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub permalink_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ScPlaylist {
    pub id: u64,
    pub title: String,
    pub permalink_url: String,
    pub artwork_url: Option<String>,
    pub track_count: Option<u32>,
    pub duration: Option<u64>,
    pub sharing: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ScPage<T> {
    Items(Vec<T>),
    Paged { collection: Vec<T> },
}

impl<T> ScPage<T> {
    pub fn into_items(self) -> Vec<T> {
        match self {
            Self::Items(v) | Self::Paged { collection: v } => v,
        }
    }
}


#[derive(Deserialize)]
pub struct ScUserRef {
    pub username: Option<String>,
}

#[derive(Deserialize)]
pub struct ScTrack {
    pub id: u64,
    pub title: Option<String>,
    pub duration: Option<u64>,
    pub permalink_url: Option<String>,
    pub artwork_url: Option<String>,
    pub access: Option<String>,
    pub user: Option<ScUserRef>,
}

#[derive(Deserialize)]
pub struct ScPlaylistDetail {
    pub id: u64,
    pub title: String,
    pub permalink_url: String,
    pub artwork_url: Option<String>,
    pub tracks: Option<Vec<ScTrack>>,
}
