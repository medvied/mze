use std::error;

use serde;

use crate::{Container, EntityId, Renderer};

pub mod web;

#[derive(Debug, serde::Deserialize)]
pub struct EntityPath {
    pub container_id_lo: Option<u64>,
    pub container_id_hi: Option<u64>,
    pub id_lo: u64,
    pub id_hi: u64,
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
        Some(EntityId::new(self.container_id_lo?, self.container_id_hi?))
    }

    pub fn get_id(&self) -> EntityId {
        EntityId::new(self.id_lo, self.id_hi)
    }
}
