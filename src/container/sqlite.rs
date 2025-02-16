use std::{
    collections::{HashMap, HashSet},
    error, iter,
};

use log::{debug, error};

use rusqlite;

use thiserror::Error;

use crate::{
    Container, ContainerTransaction, EntityId, Link, Record, SearchQuery,
    SearchResult, SearchResultRecord, SearchResultTag, ENTITY_ID_START,
};

#[derive(Error, Debug)]
pub enum ContainerSqliteError {
    #[error("can't open sqlite connection")]
    CantOpenSqliteConnection { uri: String, err: rusqlite::Error },
    #[error("failed to create table using sql={sql}")]
    FailedToCreateTable { sql: String },
    #[error("sqlite query failed: sql={sql} err={err}")]
    SqliteQueryFailed { sql: String, err: rusqlite::Error },
    #[error("sqlite conn.prepare() failed: sql={sql} err={err}")]
    SqliteConnPrepareFailed { sql: String, err: rusqlite::Error },
    #[error("sqlite statement.query_map() failed: err={err}")]
    SqliteQueryMapFailed { err: rusqlite::Error },
    #[error("too many rows for a single record: eid={eid:?}")]
    TooManyRowsForARecord { eid: EntityId },
    #[error("error retrieving record data: err={err}")]
    ErrorRetrievingRecordData { err: rusqlite::Error },
    #[error("error executing prepared statement: sql={sql} err={err}")]
    ErrorExecutingStatement { sql: String, err: rusqlite::Error },
    #[error("failed to insert 1 entry: sql={sql} inserted={nr_inserted}")]
    FailedToInsert1Entry { sql: String, nr_inserted: usize },
    #[error("failed to get record max version: sql={sql} err={err}")]
    FailedToGetRecordMaxVer { sql: String, err: rusqlite::Error },
    #[error("duplicate tags in container: eid={eid:?} tag={tag}")]
    DuplicateTagsInContainer { eid: EntityId, tag: String },
    #[error(
        "duplicate attributes in container: \
         eid={eid:?} key={key} value={value}"
    )]
    DuplicateAttributesInContainer {
        eid: EntityId,
        key: String,
        value: String,
    },
    #[error("conn.transaction() failed: err={err}")]
    BeginTransactionFailed { err: rusqlite::Error },
    #[error("tx.commit() failed: err={err}")]
    CommitTransactionFailed { err: rusqlite::Error },
    #[error("tx.rollback() failed: err={err}")]
    RollbackTransactionFailed { err: rusqlite::Error },
}

pub struct ContainerSqlite {
    conn: rusqlite::Connection,
}

pub struct ContainerSqliteTransaction<'a> {
    tx: rusqlite::Transaction<'a>,
}

impl ContainerSqlite {
    fn statements_execute(
        &self,
        statements: &[&str],
    ) -> Result<(), Box<dyn error::Error>> {
        for sql in statements {
            debug!("statement_execute(): sql={sql}");
            let result = self.conn.execute(sql, ());
            if result.is_err() {
                return Err(Box::new(
                    ContainerSqliteError::FailedToCreateTable {
                        sql: sql.to_string(),
                    },
                ));
            }
        }
        Ok(())
    }
}

impl Container for ContainerSqlite {
    fn new(uri: &str) -> Result<Self, Box<dyn error::Error>> {
        let conn = if uri.is_empty() {
            rusqlite::Connection::open_in_memory()
        } else {
            rusqlite::Connection::open(uri)
        };
        match conn {
            Ok(conn) => Ok(ContainerSqlite { conn }),
            Err(e) => {
                Err(Box::new(ContainerSqliteError::CantOpenSqliteConnection {
                    uri: uri.to_string(),
                    err: e,
                }))
            }
        }
    }

    /// TODO use NOT NULL here and check for NULL
    fn create(&self) -> Result<(), Box<dyn error::Error>> {
        let statements: &[&str] = &[
            "CREATE TABLE records(\
                id INTEGER, \
                data BLOB\
            ) STRICT;",
            "CREATE TABLE tags(\
                id INTEGER, \
                tag TEXT\
            ) STRICT;",
            "CREATE TABLE attributes(\
                id INTEGER, \
                key TEXT, \
                value TEXT\
            ) STRICT;",
            "CREATE TABLE links(\
                id INTEGER, \
                is_to INTEGER, \
                record_id INTEGER\
            ) STRICT;",
        ];
        self.statements_execute(statements)
    }

    fn destroy(&self) -> Result<(), Box<dyn error::Error>> {
        let statements: &[&str] = &[
            "DROP TABLE links;",
            "DROP TABLE attributes;",
            "DROP TABLE tags;",
            "DROP TABLE records;",
        ];
        self.statements_execute(statements)
    }

    fn load(&self, _uri: String) {
        todo!();
    }

    fn save(&self, _uri: String) {
        todo!();
    }

    fn begin_transaction(
        &mut self,
    ) -> Result<Box<dyn ContainerTransaction + '_>, Box<dyn error::Error>>
    {
        let tx = self.conn.transaction();
        match tx {
            Ok(tx) => Ok(Box::new(ContainerSqliteTransaction { tx })),
            Err(err) => {
                Err(Box::new(ContainerSqliteError::BeginTransactionFailed {
                    err,
                }))
            }
        }
    }
}

impl ContainerSqliteTransaction<'_> {
    fn eid_next(&self) -> Result<EntityId, Box<dyn error::Error>> {
        let all_record_ids = self.record_get_all_ids()?;
        let all_link_ids = self.link_get_all_ids()?;
        Ok(iter::chain(all_record_ids, all_link_ids)
            .max()
            .map(|eid| eid.add_1())
            .unwrap_or(ENTITY_ID_START))
    }

    fn tags_all(&self) -> Vec<String> {
        Vec::new()
    }
}

impl ContainerTransaction for ContainerSqliteTransaction<'_> {
    fn search(
        &self,
        search_query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn error::Error>> {
        Ok(match search_query {
            SearchQuery::Tags(tags) => {
                let return_all_tags = tags.tag_substrings.is_empty();
                HashSet::<String>::from_iter(self.tags_all())
                    .into_iter()
                    .filter_map(|tag| {
                        if return_all_tags
                            || tags
                                .tag_substrings
                                .iter()
                                .any(|s| tag.contains(s))
                        {
                            Some(SearchResult::Tag(SearchResultTag { tag }))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            SearchQuery::Attributes(_attributes) => Vec::new(),
            SearchQuery::RecordsAndLinks(_records_and_links) => self
                .record_get_all_ids()?
                .iter()
                .map(|record_id| {
                    SearchResult::Record(SearchResultRecord {
                        record_id: *record_id,
                    })
                })
                .collect(),
        })
    }

    fn commit(self: Box<Self>) -> Result<(), Box<dyn error::Error>> {
        let result = self.tx.commit();
        match result {
            Ok(ok) => Ok(ok),
            Err(err) => {
                Err(Box::new(ContainerSqliteError::CommitTransactionFailed {
                    err,
                }))
            }
        }
    }

    fn rollback(self: Box<Self>) -> Result<(), Box<dyn error::Error>> {
        let result = self.tx.rollback();
        match result {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Box::new(
                ContainerSqliteError::RollbackTransactionFailed { err },
            )),
        }
    }

    fn tags_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<String>, Box<dyn error::Error>> {
        let sql = "SELECT tag \
             FROM tags \
             WHERE \
             id = ?\
             ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let rows = statement.query_map((eid.id() as i64,), |row| {
            row.get::<&str, String>("tag")
        });
        if let Err(err) = rows {
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut tags = HashSet::new();
        for row in rows.unwrap() {
            match row {
                Ok(tag) => {
                    // TODO find a way to not to clone the tag
                    // for the error message
                    let inserted = tags.insert(tag.clone());
                    if !inserted {
                        return Err(Box::new(
                            ContainerSqliteError::DuplicateTagsInContainer {
                                eid: *eid,
                                tag,
                            },
                        ));
                    }
                }
                Err(err) => {
                    return Err(Box::new(
                        ContainerSqliteError::ErrorRetrievingRecordData {
                            err,
                        },
                    ));
                }
            }
        }
        Ok(tags.into_iter().collect())
    }

    fn tags_put(
        &mut self,
        eid: &EntityId,
        tags: &[String],
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "INSERT INTO tags(id, tag) \
                   VALUES(?, ?);";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        for tag in tags {
            let nr_inserted = statement.execute((eid.id() as i64, tag));
            if let Err(err) = nr_inserted {
                return Err(Box::new(
                    ContainerSqliteError::ErrorExecutingStatement {
                        sql: sql.to_string(),
                        err,
                    },
                ));
            }
        }
        Ok(())
    }

    fn tags_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "DELETE FROM tags WHERE \
                   id = ?\
                   ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let nr_deleted = statement.execute((eid.id() as i64,));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        debug!("tags_del(): eid={eid:?} nr_deleted={}", nr_deleted.unwrap());
        Ok(())
    }

    fn attributes_get(
        &self,
        eid: &EntityId,
    ) -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
        let sql = "SELECT key, value \
             FROM attributes \
             WHERE \
             id = ?\
             ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let rows = statement.query_map((eid.id() as i64,), |row| {
            let key = row.get::<&str, String>("key");
            let value = row.get::<&str, String>("value");
            Ok((key, value))
        });
        if let Err(err) = rows {
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut attributes = HashMap::new();
        for row in rows.unwrap() {
            match row {
                Ok((key, value)) => {
                    if let Err(err) = key {
                        return Err(Box::new(
                            ContainerSqliteError::SqliteQueryMapFailed { err },
                        ));
                    }
                    let key = key.unwrap();
                    if let Err(err) = value {
                        return Err(Box::new(
                            ContainerSqliteError::SqliteQueryMapFailed { err },
                        ));
                    }
                    let value = value.unwrap();
                    // TODO find a way to not to clone the key and value
                    // for the error message
                    let old = attributes.insert(key.clone(), value.clone());
                    if old.is_some() {
                        return Err(Box::new(
                            ContainerSqliteError::
                            DuplicateAttributesInContainer {
                                eid: *eid,
                                key,
                                value,
                            },
                        ));
                    }
                }
                Err(err) => {
                    return Err(Box::new(
                        ContainerSqliteError::ErrorRetrievingRecordData {
                            err,
                        },
                    ));
                }
            }
        }
        Ok(attributes.into_iter().collect())
    }

    fn attributes_put(
        &mut self,
        eid: &EntityId,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "INSERT INTO attributes(id, key, value) \
                   VALUES(?, ?, ?);";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        for (key, value) in attributes {
            let nr_inserted = statement.execute((eid.id() as i64, key, value));
            if let Err(err) = nr_inserted {
                return Err(Box::new(
                    ContainerSqliteError::ErrorExecutingStatement {
                        sql: sql.to_string(),
                        err,
                    },
                ));
            }
        }
        Ok(())
    }

    fn attributes_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "DELETE FROM attributes WHERE \
                   id = ?\
                   ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let nr_deleted = statement.execute((eid.id() as i64,));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        debug!(
            "attributes_del(): eid={eid:?} nr_deleted={}",
            nr_deleted.unwrap()
        );
        Ok(())
    }

    fn record_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Record>, Box<dyn error::Error>> {
        let sql = "SELECT data \
             FROM records \
             WHERE \
             id = ?\
             ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let rows = statement.query_map((eid.id() as i64,), |row| {
            row.get::<&str, Vec<u8>>("data")
        });
        if let Err(err) = rows {
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let rows: Vec<_> = rows.unwrap().collect();
        if rows.len() > 1 {
            return Err(Box::new(
                ContainerSqliteError::TooManyRowsForARecord { eid: *eid },
            ));
        }
        if rows.is_empty() {
            return Ok(None);
        }
        let first = rows.into_iter().nth(0).unwrap();
        let data = match first {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::ErrorRetrievingRecordData { err },
                ));
            }
        };
        let record = Record {
            ta: self.tags_and_attributes_get(eid)?,
            data: Some(data),
        };
        Ok(Some(record))
    }

    fn record_put(
        &mut self,
        eid: &Option<EntityId>,
        record: &Record,
    ) -> Result<EntityId, Box<dyn error::Error>> {
        let eid = match eid {
            None => self.eid_next()?,
            Some(eid) => *eid,
        };
        self.tags_and_attributes_put(&eid, &record.ta)?;
        let sql = "INSERT INTO records(id, data) \
                   VALUES(?, ?);";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let nr_inserted = statement.execute((eid.id() as i64, &record.data));
        if let Err(err) = nr_inserted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        let nr_inserted = nr_inserted.unwrap();
        if nr_inserted != 1 {
            return Err(Box::new(
                ContainerSqliteError::FailedToInsert1Entry {
                    sql: sql.to_string(),
                    nr_inserted,
                },
            ));
        }
        Ok(eid)
    }

    fn record_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>> {
        self.tags_and_attributes_del(eid)?;
        let sql = "DELETE FROM records WHERE \
                   id = ?\
                   ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let nr_deleted = statement.execute((eid.id() as i64,));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        // TODO check nr_deleted <= 1
        Ok(nr_deleted.unwrap() > 0)
    }

    fn record_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        let sql = "SELECT id \
                   FROM records;";
        debug!("tx.prepare(): sql={sql}");
        let statement = self.tx.prepare(sql);
        if let Err(err) = statement {
            return Err(Box::new(
                ContainerSqliteError::SqliteConnPrepareFailed {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        let mut statement = statement.unwrap();
        let rows = statement.query_map((), |row| row.get::<&str, i64>("id"));
        if let Err(err) = rows {
            debug!("err={err}");
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut vids = Vec::<EntityId>::new();
        for row in rows.unwrap() {
            match row {
                Ok(id) => vids.push(EntityId::new(id as u64)),
                Err(err) => {
                    return Err(Box::new(
                        ContainerSqliteError::ErrorRetrievingRecordData {
                            err,
                        },
                    ));
                }
            }
        }
        // TODO check for uniqueness
        Ok(vids)
    }

    fn link_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Link>, Box<dyn error::Error>> {
        let sql = "SELECT is_to, record_id \
             FROM links \
             WHERE \
             id = ?\
             ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let rows = statement.query_map((eid.id() as i64,), |row| {
            Ok((
                row.get::<&str, bool>("is_to")?,
                row.get::<&str, i64>("record_id")?,
            ))
        });
        if let Err(err) = rows {
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut from = Vec::new();
        let mut to = Vec::new();
        for row in rows.unwrap() {
            match row {
                Ok((is_to, record_id)) => {
                    let record_eid = EntityId::new(record_id as u64);
                    if is_to {
                        to.push(record_eid);
                    } else {
                        from.push(record_eid);
                    }
                }
                Err(err) => {
                    return Err(Box::new(
                        ContainerSqliteError::ErrorRetrievingRecordData {
                            err,
                        },
                    ));
                }
            }
        }
        let link = Link {
            ta: self.tags_and_attributes_get(eid)?,
            from,
            to,
        };
        Ok(Some(link))
    }

    fn link_put(
        &mut self,
        eid: &Option<EntityId>,
        link: &Link,
    ) -> Result<EntityId, Box<dyn error::Error>> {
        let eid = match eid {
            None => self.eid_next()?,
            Some(eid) => *eid,
        };
        self.tags_and_attributes_put(&eid, &link.ta)?;
        let sql = "INSERT INTO links(id, is_to, record_id) \
                   VALUES(?, ?, ?);";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        for (is_to, vec_eid) in [(false, &link.from), (true, &link.to)] {
            for record_eid in vec_eid {
                let nr_inserted = statement.execute((
                    eid.id() as i64,
                    is_to,
                    record_eid.id() as i64,
                ));
                if let Err(err) = nr_inserted {
                    return Err(Box::new(
                        ContainerSqliteError::ErrorExecutingStatement {
                            sql: sql.to_string(),
                            err,
                        },
                    ));
                }
                let nr_inserted = nr_inserted.unwrap();
                if nr_inserted != 1 {
                    return Err(Box::new(
                        ContainerSqliteError::FailedToInsert1Entry {
                            sql: sql.to_string(),
                            nr_inserted,
                        },
                    ));
                }
            }
        }
        Ok(eid)
    }

    fn link_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>> {
        self.tags_and_attributes_del(eid)?;
        let sql = "DELETE FROM links WHERE \
                   id = ?\
                   ;";
        let statement = self.tx.prepare(sql);
        let mut statement = match statement {
            Ok(ok) => ok,
            Err(err) => {
                return Err(Box::new(
                    ContainerSqliteError::SqliteConnPrepareFailed {
                        sql: sql.to_string(),
                        err,
                    },
                ))
            }
        };
        let nr_deleted = statement.execute((eid.id() as i64,));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        // TODO check that nr_deleted <= 1 and return an error if it's not
        Ok(nr_deleted.unwrap() > 0)
    }

    fn link_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        let sql = "SELECT DISTINCT id \
                   FROM links;";
        debug!("tx.prepare(): sql={sql}");
        let statement = self.tx.prepare(sql);
        if let Err(err) = statement {
            return Err(Box::new(
                ContainerSqliteError::SqliteConnPrepareFailed {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        let mut statement = statement.unwrap();
        let rows = statement.query_map((), |row| row.get::<&str, i64>("id"));
        if let Err(err) = rows {
            debug!("err={err}");
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut vids = Vec::<EntityId>::new();
        for row in rows.unwrap() {
            match row {
                Ok(id) => {
                    vids.push(EntityId::new(id as u64));
                }
                Err(err) => {
                    return Err(Box::new(
                        ContainerSqliteError::ErrorRetrievingRecordData {
                            err,
                        },
                    ));
                }
            }
        }
        // TODO check uniqueness
        Ok(vids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers;

    fn tags2hash_set(tags: &[String]) -> HashSet<&String> {
        HashSet::<&String>::from_iter(tags.iter())
    }

    fn attributes2hash_map(
        attributes: &[(String, String)],
    ) -> HashMap<&String, &String> {
        attributes.iter().map(|(s1, s2)| (s1, s2)).collect()
    }

    #[test]
    fn smoke_tags() {
        crate::app::init();

        let mut container = ContainerSqlite::new("").unwrap();
        container.create().unwrap();
        let mut test_rng = helpers::TestRng::new(1);
        let eid = helpers::random_entity_id(&mut test_rng);
        let mut tx = container.begin_transaction().unwrap();

        let tags1 = tx.tags_get(&eid).unwrap();
        assert!(tags1.is_empty());

        let tags = helpers::random_tags(&mut test_rng);
        tx.tags_put(&eid, &tags).unwrap();

        let tags1 = tx.tags_get(&eid).unwrap();
        assert_eq!(tags2hash_set(&tags1), tags2hash_set(&tags));

        tx.tags_del(&eid).unwrap();

        let tags1 = tx.tags_get(&eid).unwrap();
        assert!(tags1.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();
    }

    #[test]
    fn smoke_attributes() {
        crate::app::init();

        let mut container = ContainerSqlite::new("").unwrap();
        container.create().unwrap();
        let mut test_rng = helpers::TestRng::new(1);
        let eid = helpers::random_entity_id(&mut test_rng);
        let mut tx = container.begin_transaction().unwrap();

        let attributes1 = tx.attributes_get(&eid).unwrap();
        assert!(attributes1.is_empty());

        let attributes = helpers::random_attributes(&mut test_rng);
        tx.attributes_put(&eid, &attributes).unwrap();

        let attributes1 = tx.attributes_get(&eid).unwrap();
        assert_eq!(
            attributes2hash_map(&attributes1),
            attributes2hash_map(&attributes),
        );

        tx.attributes_del(&eid).unwrap();

        let attributes1 = tx.attributes_get(&eid).unwrap();
        assert!(attributes1.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();
    }

    #[test]
    fn smoke_record() {
        crate::app::init();

        let mut container = ContainerSqlite::new("").unwrap();
        container.create().unwrap();
        let mut test_rng = helpers::TestRng::new(1);
        let eid = helpers::random_entity_id(&mut test_rng);
        let mut tx = container.begin_transaction().unwrap();

        let all_ids = tx.record_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let record = helpers::random_record(&mut test_rng);
        let eid1 = tx.record_put(&Some(eid), &record).unwrap();
        assert_eq!(eid1, eid);

        let record1 = tx.record_get(&eid).unwrap().unwrap();
        assert_eq!(
            tags2hash_set(&record1.ta.tags),
            tags2hash_set(&record.ta.tags)
        );
        assert_eq!(
            attributes2hash_map(&record1.ta.attributes),
            attributes2hash_map(&record.ta.attributes)
        );
        assert_eq!(record1.data, record.data);

        let all_ids = tx.record_get_all_ids().unwrap();
        assert_eq!(all_ids.len(), 1);
        assert_eq!(all_ids[0], eid);

        let deleted = tx.record_del(&eid).unwrap();
        assert!(deleted);

        let record1 = tx.record_get(&eid).unwrap();
        assert!(record1.is_none());

        let all_ids = tx.record_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();

        // TODO test record and link put() with eid=None
    }

    #[test]
    fn smoke_link() {
        crate::app::init();

        let mut container = ContainerSqlite::new("").unwrap();
        container.create().unwrap();
        let mut test_rng = helpers::TestRng::new(1);
        let mut tx = container.begin_transaction().unwrap();

        let record_eid1 = helpers::random_entity_id(&mut test_rng);
        let record1 = helpers::random_record(&mut test_rng);
        let _ = tx.record_put(&Some(record_eid1), &record1).unwrap();
        let record_eid2 = helpers::random_entity_id(&mut test_rng);
        let record2 = helpers::random_record(&mut test_rng);
        let _ = tx.record_put(&Some(record_eid2), &record2).unwrap();

        let eid = helpers::random_entity_id(&mut test_rng);

        let all_ids = tx.link_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let link = Link {
            ta: helpers::random_tags_and_attributes(&mut test_rng),
            from: vec![record_eid1],
            to: vec![record_eid2],
        };

        let eid1 = tx.link_put(&Some(eid), &link).unwrap();
        assert_eq!(eid1, eid);

        let link1 = tx.link_get(&eid).unwrap().unwrap();
        assert_eq!(
            tags2hash_set(&link1.ta.tags),
            tags2hash_set(&link.ta.tags)
        );
        assert_eq!(
            attributes2hash_map(&link1.ta.attributes),
            attributes2hash_map(&link.ta.attributes)
        );
        assert_eq!(link1.from, link.from);
        assert_eq!(link1.to, link.to);

        let all_ids = tx.link_get_all_ids().unwrap();
        assert_eq!(all_ids.len(), 1, "all_ids={:?}", all_ids);
        assert_eq!(all_ids[0], eid);

        let deleted = tx.link_del(&eid).unwrap();
        assert!(deleted);
        let all_ids = tx.link_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let deleted = tx.record_del(&record_eid1).unwrap();
        assert!(deleted);
        let deleted = tx.record_del(&record_eid2).unwrap();
        assert!(deleted);
        let record_all_ids = tx.record_get_all_ids().unwrap();
        assert!(record_all_ids.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();
    }
}
