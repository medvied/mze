// SPDX-License-Identifier: AGPL-3.0-or-later
/*
    mze - personal knowledge database
    Copyright (C) 2024, 2025  Maksym Medvied

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
