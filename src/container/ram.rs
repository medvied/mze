use std::{
    collections::{HashMap, HashSet},
    error, iter,
};

use log::{debug, error};

use crate::{
    Container, ContainerTransaction, EntityId, Link, Record, SearchQuery,
    SearchResult, ENTITY_ID_START,
};

pub struct ContainerRam {}
pub struct ContainerRamTransaction {}

impl Container for ContainerRam {
    fn new(uri: &str) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn create(&mut self) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn destroy(&mut self) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn load(&mut self, _uri: String) {}

    fn save(&mut self, _uri: String) {}

    fn begin_transaction(
        &mut self,
    ) -> Result<Box<dyn ContainerTransaction + '_>, Box<dyn error::Error>>
    {
        Ok(Box::new(ContainerRamTransaction {}))
    }
}

impl ContainerTransaction for ContainerRamTransaction {
    fn commit(self: Box<Self>) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn rollback(self: Box<Self>) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn search(
        &self,
        search_query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn error::Error>> {
        Ok(Vec::new())
    }

    fn tags_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<String>, Box<dyn error::Error>> {
        Ok(Vec::new())
    }

    fn tags_put(
        &mut self,
        eid: &EntityId,
        tags: &[String],
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn tags_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn tags_all(&self) -> Result<Vec<String>, Box<dyn error::Error>> {
        Ok(Vec::new())
    }

    fn attributes_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
        Ok(Vec::new())
    }

    fn attributes_put(
        &mut self,
        eid: &EntityId,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn attributes_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn attributes_all(
        &self,
    ) -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
        Ok(Vec::new())
    }

    fn record_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Record>, Box<dyn error::Error>> {
        Ok(None)
    }

    fn record_put(
        &mut self,
        eid: &Option<EntityId>,
        record: &Record,
    ) -> Result<EntityId, Box<dyn error::Error>> {
        Ok(EntityId::new(0))
    }

    fn record_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>> {
        Ok(false)
    }

    fn record_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        Ok(Vec::new())
    }

    fn link_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Link>, Box<dyn error::Error>> {
        Ok(None)
    }
    fn link_put(
        &mut self,
        eid: &Option<EntityId>,
        link: &Link,
    ) -> Result<EntityId, Box<dyn error::Error>> {
        Ok(EntityId::new(0))
    }

    fn link_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>> {
        Ok(false)
    }

    fn link_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        Ok(Vec::new())
    }
}
