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

use std::{collections::HashSet, error};

use crate::{ContainerTransaction, SearchQuery, SearchResult};

pub struct Search {}

impl Search {
    pub fn search(
        tx: &(impl ContainerTransaction + ?Sized),
        search_query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn error::Error>> {
        Ok(match search_query {
            SearchQuery::Tags(tags) => {
                let return_all_tags = tags.is_empty();
                HashSet::<String>::from_iter(tx.tags_all()?)
                    .into_iter()
                    .filter_map(|tag| {
                        if return_all_tags || tags.check(&tag) {
                            Some(SearchResult::new_tag(tag))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            SearchQuery::Attributes(attributes) => {
                let return_all_attributes = attributes.is_empty();
                HashSet::<(String, String)>::from_iter(tx.attributes_all()?)
                    .into_iter()
                    .filter_map(|(key, value)| {
                        if return_all_attributes
                            || attributes.check(&key, &value)
                        {
                            Some(SearchResult::new_attribute(key, value))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            SearchQuery::RecordsAndLinks(records_and_links) => tx
                .record_get_all_ids()?
                .into_iter()
                .filter_map(|record_id| {
                    let record = tx.record_get(&record_id);
                    match record {
                        Ok(record) => match record {
                            Some(record) => {
                                if records_and_links.check_record(&record) {
                                    Some(SearchResult::new_record(record_id))
                                } else {
                                    None
                                }
                            }
                            None => Some(SearchResult::new_error(format!(
                                "No such record with record_id={record_id:?}"
                            ))),
                        },
                        Err(err) => Some(SearchResult::new_error(format!(
                            "Failed to get record \
                                        with record_id={record_id:?}: {err}"
                        ))),
                    }
                })
                .chain(tx.link_get_all_ids()?.into_iter().filter_map(
                    |link_id| {
                        let link = tx.link_get(&link_id);
                        match link {
                            Ok(link) => match link {
                                Some(link) => {
                                    if records_and_links.check_link(&link) {
                                        Some(SearchResult::new_link(link_id))
                                    } else {
                                        None
                                    }
                                }
                                None => {
                                    Some(SearchResult::new_error(format!(
                                        "No such link with link_id={link_id:?}"
                                )))
                                }
                            },
                            Err(err) => {
                                Some(SearchResult::new_error(format!(
                                    "Failed to get link \
                                        with link_id={link_id:?}: {err}"
                                )))
                            }
                        }
                    },
                ))
                .collect(),
        })
    }
}
