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
 */

use std::collections::{
    HashMap,
    HashSet,
};

pub struct EntityId {
    /// Container-unique entity id.
    pub id: u128,
    /// Monotonically increasing sequence. Starts from 1.
    /// [`ENTITY_VERSION_LATEST`] means "the latest version".
    ///
    /// [`ENTITY_VERSION_LATEST`]: ENTITY_VERSION_LATEST
    pub version: u64,
}

pub const ENTITY_VERSION_LATEST: u64 = 0;

pub struct Entity {
    pub tags: HashSet<String>,
    pub attrs: HashMap<String, String>,
}

pub struct Record {
    pub entity: Entity,
    pub data: Option<Vec<u8>>,
}

pub struct Link {
    pub entity: Entity,
    pub from: Vec<EntityId>,
    pub to: Vec<EntityId>,
}

pub struct SearchResult {
    pub records: Vec<EntityId>,
    pub links: Vec<EntityId>,
}

pub trait Container {
    fn load(self);
    fn save(self);

    fn search(self, query: String) -> SearchResult;

    fn get_record(self, eid: EntityId) -> Record;
    fn get_link(self, eid: EntityId) -> Link;
    fn put_record(self, eid: EntityId, record: &Record);
    fn put_link(self, eid: EntityId, link: &Link);
    fn del_entity(self, eid: EntityId);

    fn get_records(self, record_eids: Vec<EntityId>) -> Vec<Record>;
    fn get_links(self, link_eids: Vec<EntityId>) -> Vec<Link>;

    fn get_all_entities_with_all_versions(self) -> Vec<EntityId>;

    /// Returns latest versions of every record.
    fn get_all_records(self) -> Vec<EntityId>;
    /// See [`get_all_records`].
    ///
    /// [`get_all_records`]: Container::get_all_records
    fn get_all_links(self) -> Vec<EntityId>;
}

pub struct Registry {
    pub containers: Vec<Box<dyn Container>>,
}

pub trait Renderer {
    fn run(self);
}
