use std::error;

use serde;

use crate::{Container, EntityId, Renderer};

pub mod web;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EntityPath {
    pub container_id: Option<u64>,
    pub id: u64,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EntitiesPath {
    pub container_id: Option<u64>,
    pub id: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UriSearchQuery {
    pub q: String,
}

pub fn new(
    name: &str,
    uri: &str,
    container: Box<dyn Container + Send>,
) -> Result<Box<dyn Renderer>, Box<dyn error::Error>> {
    Ok(Box::new(match name {
        "web" => web::RendererWeb::new(uri, container)?,
        _ => panic!("renderer::new(): unknown name={name}"),
    }))
}

impl EntityPath {
    pub fn get_container_id(&self) -> Option<EntityId> {
        Some(EntityId::new(self.container_id?))
    }

    pub fn get_id(&self) -> EntityId {
        EntityId::new(self.id)
    }
}

impl EntitiesPath {
    pub fn get_entity_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        let ids: Result<Vec<u64>, _> =
            self.id.split(",").map(|s| s.parse::<u64>()).collect();
        let ids: Vec<EntityId> =
            ids?.into_iter().map(|id| EntityId { id }).collect();
        if ids.is_empty() {
            Err("the list of ids is empty".into())
        } else {
            Ok(ids)
        }
    }
}
