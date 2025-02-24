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
