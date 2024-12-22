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
}

pub struct EntityIdVer {
    // see EntityId::id
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

    fn record_get(self, eidv: EntityIdVer) -> Record;
    fn record_put(self, eid: EntityId, record: Record) -> EntityIdVer;
    fn record_vec_get(self, vidv: Vec<EntityIdVer>) -> Vec<Record>;
    fn record_vec_put(self, idvs_and_records: Vec<(EntityIdVer, Record)>);
    /// Returns latest versions of every record.
    fn record_get_all(self) -> Vec<EntityIdVer>;

    fn link_get(self, eidv: EntityIdVer) -> Link;
    fn link_put(self, eid: EntityId, link: Link) -> EntityIdVer;
    fn link_vec_get(self, vidv: Vec<EntityIdVer>) -> Vec<Link>;
    fn link_vec_put(self, idvs_and_links: Vec<(EntityIdVer, Link)>);
    /// Returns latest versions of every link.
    fn link_get_all(self) -> Vec<EntityIdVer>;

    fn entity_del(self, eidv: EntityIdVer);
    fn entity_vec_del(self, vidv: Vec<EntityIdVer>);
    fn entity_get_versions(self, eid: EntityId) -> Vec<EntityIdVer>;
    fn entity_vec_get_versions(self, vid: Vec<EntityId>)
        -> Vec<Vec<EntityIdVer>>;
}

pub struct Registry {
    pub containers: Vec<Box<dyn Container>>,
}

pub trait Renderer {
    fn run(self);
}
