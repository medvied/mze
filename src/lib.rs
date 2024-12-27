/*
   Copyright 2024 Maksym Medvied

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

/*!
 * Reserved tags:
 *
 * - tombstone
 *
 * Reserved attributes
 *
 * - type: record | link
 *
 * TODO log debug messages and errors consistently
 */

// to make thread::current().id().as_u64() work
#![feature(thread_id_value)]

pub mod app;
pub mod container;
// rusrc adds the following message if the name is test or test_helpers
//
// help: if this is a test module, consider adding a `#[cfg(test)]`
// to the containing module
pub mod helpers;

use std::{
    collections::{HashMap, HashSet},
    error,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityId {
    /// Container-unique entity id.
    pub id: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityIdVer {
    // see EntityId::id
    pub id: u128,
    /// Version.
    /// Monotonically increasing sequence. Starts from 1.
    /// [`ENTITY_VERSION_LATEST`] means "the latest version".
    ///
    /// [`ENTITY_VERSION_LATEST`]: ENTITY_VERSION_LATEST
    pub ver: u64,
}

pub const ENTITY_VERSION_LATEST: u64 = 0;

#[derive(Debug, Eq, PartialEq)]
pub struct TagsAndAttrs {
    pub tags: HashSet<String>,
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Record {
    pub ta: TagsAndAttrs,
    pub data: Option<Vec<u8>>,
}

pub struct Link {
    pub ta: TagsAndAttrs,
    pub from: Vec<EntityId>,
    pub to: Vec<EntityId>,
}

pub struct SearchResult {
    pub records: Vec<EntityIdVer>,
    pub links: Vec<EntityIdVer>,
}

pub trait ContainerTransaction {
    fn commit(self) -> Result<(), Box<dyn error::Error>>;
    fn rollback(self) -> Result<(), Box<dyn error::Error>>;

    fn tags_get(
        &self,
        eidv: &EntityIdVer,
    ) -> Result<HashSet<String>, Box<dyn error::Error>>;
    fn tags_put(
        &mut self,
        eidv: &EntityIdVer,
        tags: &HashSet<String>,
    ) -> Result<(), Box<dyn error::Error>>;
    fn tags_del(
        &mut self,
        eidv: &EntityIdVer,
    ) -> Result<(), Box<dyn error::Error>>;

    fn attrs_get(
        &self,
        eidv: &EntityIdVer,
    ) -> Result<HashMap<String, String>, Box<dyn error::Error>>;
    fn attrs_put(
        &mut self,
        eidv: &EntityIdVer,
        attrs: &HashMap<String, String>,
    ) -> Result<(), Box<dyn error::Error>>;
    fn attrs_del(
        &mut self,
        eidv: &EntityIdVer,
    ) -> Result<(), Box<dyn error::Error>>;

    fn record_get(
        &self,
        eidv: &EntityIdVer,
    ) -> Result<Option<Record>, Box<dyn error::Error>>;
    fn record_put(
        &mut self,
        eid: &EntityId,
        record: &Record,
    ) -> Result<EntityIdVer, Box<dyn error::Error>>;
    fn record_del(
        &mut self,
        eidv: &EntityIdVer,
    ) -> Result<(), Box<dyn error::Error>>;
    /// Returns latest versions of every record.
    fn record_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>>;
    fn record_get_ver_latest(
        &self,
        eid: &EntityId,
    ) -> Result<Option<EntityIdVer>, Box<dyn error::Error>>;

    fn tags_and_attrs_get(
        &self,
        eidv: &EntityIdVer,
    ) -> Result<TagsAndAttrs, Box<dyn error::Error>> {
        Ok(TagsAndAttrs {
            tags: self.tags_get(eidv)?,
            attrs: self.attrs_get(eidv)?,
        })
    }

    fn tags_and_attrs_put(
        &mut self,
        eidv: &EntityIdVer,
        ta: &TagsAndAttrs,
    ) -> Result<(), Box<dyn error::Error>> {
        self.tags_put(eidv, &ta.tags)?;
        self.attrs_put(eidv, &ta.attrs)
    }

    fn tags_and_attrs_del(
        &mut self,
        eidv: &EntityIdVer,
    ) -> Result<(), Box<dyn error::Error>> {
        self.tags_del(eidv)?;
        self.attrs_del(eidv)
    }
}

pub trait Container {
    type Transaction<'a>
    where
        Self: 'a;

    fn create(&self) -> Result<(), Box<dyn error::Error>>;
    fn destroy(&self) -> Result<(), Box<dyn error::Error>>;
    fn load(&self, uri: String);
    fn save(&self, uri: String);

    fn begin_transaction(
        &mut self,
    ) -> Result<Self::Transaction<'_>, Box<dyn error::Error>>;

    fn search(&self, query: String) -> SearchResult;
}

pub struct Registry {
    // XXX pub containers: Vec<Box<dyn Container>>,
}

pub trait Renderer {
    fn run(self);
}

impl EntityId {
    pub fn new(id_lo: u64, id_hi: u64) -> EntityId {
        EntityId {
            id: ((id_hi as u128) << 64) + (id_lo as u128),
        }
    }

    pub fn id_lo(&self) -> u64 {
        self.id as u64
    }

    pub fn id_hi(&self) -> u64 {
        (self.id >> 64) as u64
    }
}

impl EntityIdVer {
    pub fn id_lo(&self) -> u64 {
        self.id as u64
    }

    pub fn id_hi(&self) -> u64 {
        (self.id >> 64) as u64
    }

    pub fn ver(&self) -> u64 {
        self.ver
    }
}
