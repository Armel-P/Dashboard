use std::{collections::HashMap, sync::Arc};

use crate::{
    services::{
        providers::{
            open_meteo::OpenMeteo,
            soundcloud::SoundCloud
        },
        traits::Service
    }
};

#[derive(Clone)]
pub struct Registry(Arc<HashMap<&'static str, Arc<dyn Service>>>);

impl Registry {
    pub fn new() -> Self {
        let services: Vec<Arc<dyn Service>> = vec![
            Arc::new(OpenMeteo),
            Arc::new(SoundCloud),
        ];
        Self(Arc::new(services.into_iter().map(|s| (s.name(), s)).collect()))
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Service>> {
        self.0.get(name).cloned()
    }

    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn Service>> {
        self.0.values()
    }
}