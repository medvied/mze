use std::error;

use serde;

use crate::{Container, EntityId, Renderer};

pub mod web;

#[derive(Debug, serde::Deserialize)]
pub struct EntityPath {
    pub container_id: Option<u64>,
    pub id: u64,
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
