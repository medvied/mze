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
 * Design highlights
 *
 * - record, link and container are entities
 * - link is directional
 * - link to and from are unordered sets
 * - there may be more than 1 link between 2 records
 * - tags: unordered set of UTF-8 strings
 * - attributes: unordered key:value map, both key and value are UTF-8 strings
 * - types for:
 *
 *   - tags
 *   - attributes (example: time format)
 *   - links (example: one image is a thumbnail of another)
 *   - records (example: outline is a record that has a tree-like structure
 *     that reference other records)
 *
 *  - container may contain records, links and other containers
 *
 *  Test data
 *
 *  - Linux kernel tree: files, functions, call graph
 *
 *  Search query
 *
 *  The search is done in the following way:
 *
 * - the search string is split into whitespace-separated tokens
 *   (for the purpose of the further description: "xxx" token is 3 "x" chars,
 *   double quotes are not included);
 * - if there is "#" token in the search string, then this is a search for tags
 *   only:
 *
 *   - if there are no tokens other than "#" then the result is all tags in the
 *     container;
 *   - "#" prefix is removed from every token (only once), empty tokens are
 *     removed, and the search result is a set of tags that have any of the
 *     tokens as a substring;
 *   - the search ends here.
 *
 *  - if there is "#=" token in the search string, then this is a search for
 *    attributes:
 *
 *    - if the are no tokens other than "#=" then the result is a set of all
 *      keys and values in the container;
 *    - "#" prefix is removed from every token (only once), empty tockens are
 *      removed, the result is split into key and value on the first "=";
 *    - for each key=value there is a set of all keys and values where the key
 *      from key=value is a substring of the key and the value from key=value
 *      is a substring of the value. For the purpose of building the sets an
 *      empty string is a subset of any string;
 *    - the search result is a union of all sets described in the previous
 *      paragraph;
 *    - the search ends here.
 *
 *  - otherwise, this is a search for links and records:
 *
 *    - first, a set of tags to look for is built. All the tokens that start
 *      with "#" and don't have "=" have the "#" prefix removed (only once),
 *      and a set of tag tokens is built;
 *    - second, a set of attributes to look for is built. All the tokents that
 *      start with "#" and have "=" inside have the "#" previx removed (only
 *      once) and a set of key:value pairs is built by splitting the result
 *      on the first "=";
 *    - the search result is a set of all records and links where each record
 *      and each link have the following properties:
 *
 *      - every tag token from the set of tag tokens is a substring of a tag of
 *        the record/link;
 *      - every key:value pair from the set of key:value pairs build from the
 *        search string has a corresponding key1:value1 pair in the record/link
 *        attributes such as key is a substring of key1 and value is a
 *        substring of value1 (an empty string is a substring of any string for
 *        the purpose of this check);
 *      - each token that doesn't start with "#" is either a substring of
 *        a tag, a substring of a key or a value or a substring of the record
 *        blob.
 *
 *    - the search ends here.
 *
 *
 *  - TODO `from:id`, `to:id` - search() looks for links with specific record
 *    ids
 *  - TODO `from:$(search query)`, `to:$(search query)` - look for links
 *    that have records from the search query
 *  - TODO $variable, $variable=value - configure search variables
 *  - TODO from:id, to:id - search tags, keys, values and text that contain
 *    the strings
 *  - TODO $variable, $variable=value
 *
 *  TODO URI
 *
 *    mze://host:port/container/id
 *    container/id
 *    id
 *
 * TODO log debug messages and errors consistently
 * TODO add # of containers/records/links/tags/attributes and the time to
 * generate the page to the footer
 * TODO root container - the container that contains all containers loaded by
 * default
 */

// to make thread::current().id().as_u64() work
#![feature(thread_id_value)]
// to make iter::chain() work
#![feature(iter_chain)]

pub mod app;
pub mod container;
pub mod renderer;
pub mod search_query;
pub use search_query::SearchQuery;
// rusrc adds the following message if the name is test or test_helpers
//
// help: if this is a test module, consider adding a `#[cfg(test)]`
// to the containing module
pub mod helpers;

use std::error;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize,
)]
pub struct EntityId {
    /// Container-unique entity id.
    pub id: u64,
}

/// Start from this EntityId by default
/// Rationale: have the same id width (in decimal) for the first ~90k ids
/// and more or less the same width for the first ~1M ids
pub const ENTITY_ID_START: EntityId = EntityId { id: 10000 };

// TODO create a data structure for tags and attributes to share
// strings between different entities (also check if Rust could do that
// automatically)

#[derive(Debug, Default, Eq, PartialEq)]
pub struct TagsAndAttributes {
    pub tags: Vec<String>,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Record {
    pub ta: TagsAndAttributes,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Link {
    pub ta: TagsAndAttributes,
    pub from: Vec<EntityId>,
    pub to: Vec<EntityId>,
}

pub struct SearchResultRecord {
    pub record_id: EntityId,
}

pub struct SearchResultLink {
    pub link_id: EntityId,
}

pub struct SearchResultTag {
    pub tag: String,
}

pub struct SearchResultAttribute {
    pub key: String,
    pub value: String,
}

pub enum SearchResult {
    Record(SearchResultRecord),
    Link(SearchResultLink),
    Tag(SearchResultTag),
    Attribute(SearchResultAttribute),
}

pub trait ContainerTransaction {
    fn commit(self: Box<Self>) -> Result<(), Box<dyn error::Error>>;
    fn rollback(self: Box<Self>) -> Result<(), Box<dyn error::Error>>;

    fn search(
        &self,
        search_query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn error::Error>>;

    fn tags_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<String>, Box<dyn error::Error>>;
    fn tags_put(
        &mut self,
        eid: &EntityId,
        tags: &[String],
    ) -> Result<(), Box<dyn error::Error>>;
    fn tags_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>>;

    fn attributes_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<(String, String)>, Box<dyn error::Error>>;
    fn attributes_put(
        &mut self,
        eid: &EntityId,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn error::Error>>;
    fn attributes_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>>;

    fn record_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Record>, Box<dyn error::Error>>;
    fn record_put(
        &mut self,
        eid: &Option<EntityId>,
        record: &Record,
    ) -> Result<EntityId, Box<dyn error::Error>>;
    fn record_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>>;
    /// Returns EntityId of every record
    fn record_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>>;

    fn link_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Link>, Box<dyn error::Error>>;
    fn link_put(
        &mut self,
        eid: &Option<EntityId>,
        link: &Link,
    ) -> Result<EntityId, Box<dyn error::Error>>;
    fn link_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>>;
    fn link_get_all_ids(&self)
        -> Result<Vec<EntityId>, Box<dyn error::Error>>;

    fn tags_and_attributes_get(
        &self,
        eid: &EntityId,
    ) -> Result<TagsAndAttributes, Box<dyn error::Error>> {
        Ok(TagsAndAttributes {
            tags: self.tags_get(eid)?,
            attributes: self.attributes_get(eid)?,
        })
    }

    fn tags_and_attributes_put(
        &mut self,
        eid: &EntityId,
        ta: &TagsAndAttributes,
    ) -> Result<(), Box<dyn error::Error>> {
        self.tags_put(eid, &ta.tags)?;
        self.attributes_put(eid, &ta.attributes)
    }

    fn tags_and_attributes_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        self.tags_del(eid)?;
        self.attributes_del(eid)
    }
}

pub trait Container {
    fn new(uri: &str) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized;

    fn create(&self) -> Result<(), Box<dyn error::Error>>;
    fn destroy(&self) -> Result<(), Box<dyn error::Error>>;
    fn load(&self, uri: String);
    fn save(&self, uri: String);

    fn begin_transaction(
        &mut self,
    ) -> Result<Box<dyn ContainerTransaction + '_>, Box<dyn error::Error>>;
}

pub trait Renderer {
    fn new(
        uri: &str,
        container: Box<dyn Container + Send>,
    ) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized;

    fn run(&mut self) -> Result<(), Box<dyn error::Error>>;
}

impl EntityId {
    pub fn new(id: u64) -> EntityId {
        EntityId { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn add_1(&self) -> Self {
        EntityId { id: self.id + 1 }
    }
}
