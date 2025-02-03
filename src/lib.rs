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
 *  Search query format
 *
 *  - the search query is a whitespace-separated list of words that have
 *    to be applicable for every search result (i.e. there is an implicit
 *    logical AND between every pair of words)
 *  - search() looks for every word from the query in the text
 *  - search() looks for every word that starts with `#`: it removes the `#`
 *    and then searches for the string in the tags as well
 *  - search() looks for every word that has `=` after `#`: it removes `#`,
 *    splits the remaining string into key and value on the first `=` and then
 *    searches for the key=value in the attributes
 *  - TODO `from:id`, `to:id` - search() looks for links with specific record
 *    ids
 *  - TODO `from:$(search query)`, `to:$(search query)` - look for links
 *    that have records from the search query
 *  - TODO $variable, $variable=value - configure search variables
 *
 *  Another way to describe search query format
 *
 *  - # - all tags (only for the tag results, for Records/links the rest of the
 *    tags query is used if present)
 *  - #= - all attributes (same as for all tags)
 *  - #tag - search for `tag` in tags and `#tag` in the text
 *  - #key=value - search for `key=value` in tags, `key`=`value` in attributes
 *    and #key=value in the text
 *  - #key= - search for `key=` in tags and `key` in the attribute keys
 *    (regardless of value for the key)
 *  - #=value - search for `=value` in tags and `value` in the attribute values
 *    (regardless of the key for attribute)
 *  - word - search for the `word` in tags, attribute keys and values and text
 *  - TODO from:id, to:id - search tags, keys, values and text that contain
 *    the strings
 *  - TODO $variable, $variable=value
 *
 *  Search algorithm
 *
 *  0. If the search string is empty - return nothing.
 *  1. Tags
 *
 *     - if # is present: find all tags - this is the result;
 *     - otherwise:
 *
 *       - for each #tag_name find all tags with tag_name as a substring, add
 *         all of them to the results;
 *       - for each text_string: same as for tag_name above.
 *
 *  2. Attributes
 *
 *     - if #= is present: find all attributes - this is the result
 *     - otherwise
 *
 *       - for #key_name=: find all keys where key_name is a substring. Add all
 *         such keys to the results (without values);
 *       - for #=value_name: similar to #key_name=;
 *       - for #key_name=value_name: find such key-value pair where key_name is
 *         a substring of the key and value_name is a substring of the value,
 *         and add all such key-values to the result;
 *       - for each text_string: look for it as a substring in the keys and
 *         values, add all matching keys, values and key-value pairs to the
 *         result.
 *
 *     - TODO define if we want keys, values or both or with some filter
 *
 *  3. Records and links
 *
 *   - for every explicit tag and/or attribute from the search box: find
 *     records/links which have all of them at the same time. Then:
 *
 *     - for links: add such links to the results;
 *     - for records: filter records futher by requiring each of them to have
 *       all text_string strings that don't start from # from the query.
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
