// SPDX-License-Identifier: AGPL-3.0-or-later
/*
    mze - personal knowledge database
    Copyright (C) 2025  Maksym Medvied

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

use std::{
    collections::{HashMap, HashSet},
    error,
};

use crate::{
    Container, ContainerError, ContainerTransaction, EntityId, Link, Record,
    RecordOrLink, TagsAndAttributes, ENTITY_ID_START,
};

pub struct ContainerRam {
    entities: HashMap<EntityId, RecordOrLink>,
}

// XXX the changes are not transactional
// TODO decide if transactions are needed here
pub struct ContainerRamTransaction<'a> {
    container_ram: &'a mut ContainerRam,
}

impl Container for ContainerRam {
    fn new(_uri: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(Self {
            entities: HashMap::new(),
        })
    }

    fn create(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.entities.clear();
        Ok(())
    }

    fn destroy(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.entities.clear();
        Ok(())
    }

    fn load(&mut self, _uri: String) {}

    fn save(&mut self, _uri: String) {}

    fn begin_transaction(
        &mut self,
    ) -> Result<Box<dyn ContainerTransaction + '_>, Box<dyn error::Error>>
    {
        Ok(Box::new(ContainerRamTransaction {
            container_ram: self,
        }))
    }
}

impl ContainerRamTransaction<'_> {
    fn ta_get(
        &self,
        eid: &EntityId,
    ) -> Result<&TagsAndAttributes, Box<dyn error::Error>> {
        match self.container_ram.entities.get(eid) {
            Some(record_or_link) => {
                Ok(record_or_link.get_tags_and_attributes())
            }
            None => {
                Err(Box::new(ContainerError::EntityNotFound { eid: *eid }))
            }
        }
    }

    fn ta_get_mut(
        &mut self,
        eid: &EntityId,
    ) -> Result<&mut TagsAndAttributes, Box<dyn error::Error>> {
        match self.container_ram.entities.get_mut(eid) {
            Some(record_or_link) => {
                Ok(record_or_link.get_tags_and_attributes_mut())
            }
            None => {
                Err(Box::new(ContainerError::EntityNotFound { eid: *eid }))
            }
        }
    }
}

impl ContainerTransaction for ContainerRamTransaction<'_> {
    fn commit(self: Box<Self>) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn rollback(self: Box<Self>) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    fn tags_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<String>, Box<dyn error::Error>> {
        Ok(self.ta_get(eid)?.tags.clone())
    }

    fn tags_put(
        &mut self,
        eid: &EntityId,
        tags: &[String],
    ) -> Result<(), Box<dyn error::Error>> {
        self.ta_get_mut(eid)?.tags = Vec::from(tags);
        Ok(())
    }

    fn tags_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        self.ta_get_mut(eid)?.tags = Vec::new();
        Ok(())
    }

    fn tags_all(&self) -> Result<Vec<String>, Box<dyn error::Error>> {
        let mut tags = HashSet::new();
        for record_or_link in self.container_ram.entities.values() {
            let ta = record_or_link.get_tags_and_attributes();
            tags.extend(ta.tags.clone());
        }
        Ok(tags.into_iter().collect())
    }

    fn attributes_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
        Ok(self.ta_get(eid)?.attributes.clone())
    }

    fn attributes_put(
        &mut self,
        eid: &EntityId,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn error::Error>> {
        self.ta_get_mut(eid)?.attributes = Vec::from(attributes);
        Ok(())
    }

    fn attributes_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        self.ta_get_mut(eid)?.attributes = Vec::new();
        Ok(())
    }

    fn attributes_all(
        &self,
    ) -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
        let mut attributes = HashSet::new();
        for record_or_link in self.container_ram.entities.values() {
            let ta = record_or_link.get_tags_and_attributes();
            attributes.extend(ta.attributes.clone());
        }
        Ok(attributes.into_iter().collect())
    }

    fn record_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Record>, Box<dyn error::Error>> {
        Ok(match self.container_ram.entities.get(eid) {
            Some(record_or_link) => match record_or_link {
                RecordOrLink::Record(record) => Some(record.clone()),
                RecordOrLink::Link(_) => {
                    return Err(Box::new(
                        ContainerError::FoundLinkInsteadOfRecord { eid: *eid },
                    ))
                }
            },
            None => None,
        })
    }

    fn record_put(
        &mut self,
        eid: &Option<EntityId>,
        record: &Record,
    ) -> Result<EntityId, Box<dyn error::Error>> {
        if let Some(eid) = eid {
            if let Some(RecordOrLink::Link(_)) =
                self.container_ram.entities.get(eid)
            {
                return Err(Box::new(
                    ContainerError::FoundLinkInsteadOfRecord { eid: *eid },
                ));
            }
        }
        let eid = if let Some(eid) = eid {
            *eid
        } else {
            ENTITY_ID_START
        };
        self.container_ram
            .entities
            .insert(eid, RecordOrLink::Record(record.clone()));
        Ok(eid)
    }

    fn record_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>> {
        if let Some(RecordOrLink::Link(_)) =
            self.container_ram.entities.get(eid)
        {
            return Err(Box::new(ContainerError::FoundLinkInsteadOfRecord {
                eid: *eid,
            }));
        }
        Ok(self.container_ram.entities.remove(eid).is_some())
    }

    fn record_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        Ok(self
            .container_ram
            .entities
            .iter()
            .filter_map(|(eid, record_or_link)| {
                if let RecordOrLink::Record(_) = record_or_link {
                    Some(*eid)
                } else {
                    None
                }
            })
            .collect())
    }

    fn link_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Link>, Box<dyn error::Error>> {
        Ok(match self.container_ram.entities.get(eid) {
            Some(record_or_link) => match record_or_link {
                RecordOrLink::Link(link) => Some(link.clone()),
                RecordOrLink::Record(_) => {
                    return Err(Box::new(
                        ContainerError::FoundRecordInsteadOfLink { eid: *eid },
                    ))
                }
            },
            None => None,
        })
    }
    fn link_put(
        &mut self,
        eid: &Option<EntityId>,
        link: &Link,
    ) -> Result<EntityId, Box<dyn error::Error>> {
        if let Some(eid) = eid {
            if let Some(RecordOrLink::Record(_)) =
                self.container_ram.entities.get(eid)
            {
                return Err(Box::new(
                    ContainerError::FoundRecordInsteadOfLink { eid: *eid },
                ));
            }
        }
        let eid = if let Some(eid) = eid {
            *eid
        } else {
            ENTITY_ID_START
        };
        self.container_ram
            .entities
            .insert(eid, RecordOrLink::Link(link.clone()));
        Ok(eid)
    }

    fn link_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>> {
        if let Some(RecordOrLink::Record(_)) =
            self.container_ram.entities.get(eid)
        {
            return Err(Box::new(ContainerError::FoundRecordInsteadOfLink {
                eid: *eid,
            }));
        }
        Ok(self.container_ram.entities.remove(eid).is_some())
    }

    fn link_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        Ok(self
            .container_ram
            .entities
            .iter()
            .filter_map(|(eid, record_or_link)| {
                if let RecordOrLink::Link(_) = record_or_link {
                    Some(*eid)
                } else {
                    None
                }
            })
            .collect())
    }
}
