use std::{
    collections::{HashMap, HashSet},
    error, iter,
};

use log::{debug, error};

use rusqlite;

use thiserror::Error;

use crate::{
    Container, ContainerTransaction, EntityId, Link, Record, SearchResult,
    ENTITY_ID_START,
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
        "duplicate attrs in container: eid={eid:?} key={key} value={value}"
    )]
    DuplicateAttrsInContainer {
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
                id_lo INTEGER, \
                id_hi INTEGER, \
                data BLOB\
            ) STRICT;",
            "CREATE TABLE tags(\
                id_lo INTEGER, \
                id_hi INTEGER, \
                tag TEXT\
            ) STRICT;",
            "CREATE TABLE attrs(\
                id_lo INTEGER, \
                id_hi INTEGER, \
                key TEXT, \
                value TEXT\
            ) STRICT;",
            "CREATE TABLE links(\
                id_lo INTEGER, \
                id_hi INTEGER, \
                is_to INTEGER, \
                r_id_lo INTEGER, \
                r_id_hi INTEGER\
            ) STRICT;",
        ];
        self.statements_execute(statements)
    }

    fn destroy(&self) -> Result<(), Box<dyn error::Error>> {
        let statements: &[&str] = &[
            "DROP TABLE links;",
            "DROP TABLE attrs;",
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

    fn search(&self, _query: String) -> SearchResult {
        todo!();
    }
}

impl ContainerSqliteTransaction<'_> {
    fn eid_stored_max(
        &self,
    ) -> Result<Option<EntityId>, Box<dyn error::Error>> {
        let all_record_ids = self.record_get_all_ids()?;
        let all_link_ids = self.link_get_all_ids()?;
        Ok(iter::chain(all_record_ids, all_link_ids).max())
    }
}

impl ContainerTransaction for ContainerSqliteTransaction<'_> {
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
    ) -> Result<HashSet<String>, Box<dyn error::Error>> {
        let sql = "SELECT tag \
             FROM tags \
             WHERE \
             id_lo = ? AND id_hi = ?\
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
        let rows = statement
            .query_map((eid.id_lo() as i64, eid.id_hi() as i64), |row| {
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
        Ok(tags)
    }

    fn tags_put(
        &mut self,
        eid: &EntityId,
        tags: &HashSet<String>,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "INSERT INTO tags(id_lo, id_hi, tag) \
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
        for tag in tags {
            let nr_inserted = statement.execute((
                eid.id_lo() as i64,
                eid.id_hi() as i64,
                tag,
            ));
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
                   id_lo = ? AND id_hi = ?\
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
        let nr_deleted =
            statement.execute((eid.id_lo() as i64, eid.id_hi() as i64));
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

    fn attrs_get(
        &self,
        eid: &EntityId,
    ) -> Result<HashMap<String, String>, Box<dyn error::Error>> {
        let sql = "SELECT key, value \
             FROM attrs \
             WHERE \
             id_lo = ? AND id_hi = ?\
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
        let rows = statement.query_map(
            (eid.id_lo() as i64, eid.id_hi() as i64),
            |row| {
                let key = row.get::<&str, String>("key");
                let value = row.get::<&str, String>("value");
                Ok((key, value))
            },
        );
        if let Err(err) = rows {
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut attrs = HashMap::new();
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
                    let old = attrs.insert(key.clone(), value.clone());
                    if old.is_some() {
                        return Err(Box::new(
                            ContainerSqliteError::DuplicateAttrsInContainer {
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
        Ok(attrs)
    }

    fn attrs_put(
        &mut self,
        eid: &EntityId,
        attrs: &HashMap<String, String>,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "INSERT INTO attrs(id_lo, id_hi, key, value) \
                   VALUES(?, ?, ?, ?);";
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
        for (key, value) in attrs {
            let nr_inserted = statement.execute((
                eid.id_lo() as i64,
                eid.id_hi() as i64,
                key,
                value,
            ));
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

    fn attrs_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "DELETE FROM attrs WHERE \
                   id_lo = ? AND id_hi = ?\
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
        let nr_deleted =
            statement.execute((eid.id_lo() as i64, eid.id_hi() as i64));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        debug!(
            "attrs_del(): eid={eid:?} nr_deleted={}",
            nr_deleted.unwrap(),
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
             id_lo = ? AND id_hi = ?\
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
        let rows = statement
            .query_map((eid.id_lo() as i64, eid.id_hi() as i64), |row| {
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
            ta: self.tags_and_attrs_get(eid)?,
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
            None => self.eid_stored_max()?.unwrap_or(ENTITY_ID_START),
            Some(eid) => *eid,
        };
        self.tags_and_attrs_put(&eid, &record.ta)?;
        let sql = "INSERT INTO records(id_lo, id_hi, data) \
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
        let nr_inserted = statement.execute((
            eid.id_lo() as i64,
            eid.id_hi() as i64,
            &record.data,
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
        Ok(eid)
    }

    fn record_del(
        &mut self,
        eid: &EntityId,
    ) -> Result<bool, Box<dyn error::Error>> {
        self.tags_and_attrs_del(eid)?;
        let sql = "DELETE FROM records WHERE \
                   id_lo = ? AND id_hi = ?\
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
        let nr_deleted =
            statement.execute((eid.id_lo() as i64, eid.id_hi() as i64));
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
        let sql = "SELECT id_hi, id_lo \
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
        let rows = statement.query_map((), |row| {
            let id_lo = row.get::<&str, i64>("id_lo");
            let id_hi = row.get::<&str, i64>("id_hi");
            Ok((id_lo, id_hi))
        });
        if let Err(err) = rows {
            debug!("err={err}");
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut vids = Vec::<EntityId>::new();
        for row in rows.unwrap() {
            match row {
                Ok((id_lo, id_hi)) => {
                    if let Err(err) = id_lo {
                        return Err(Box::new(
                            ContainerSqliteError::SqliteQueryMapFailed { err },
                        ));
                    }
                    let id_lo = id_lo.unwrap() as u64;
                    if let Err(err) = id_hi {
                        return Err(Box::new(
                            ContainerSqliteError::SqliteQueryMapFailed { err },
                        ));
                    }
                    let id_hi = id_hi.unwrap() as u64;
                    vids.push(EntityId::new(id_lo, id_hi));
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
        // TODO check for uniqueness
        Ok(vids)
    }

    fn link_get(
        &self,
        eid: &EntityId,
    ) -> Result<Option<Link>, Box<dyn error::Error>> {
        let sql = "SELECT is_to, r_id_lo, r_id_hi \
             FROM links \
             WHERE \
             id_lo = ? AND id_hi = ?\
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
        let rows = statement.query_map(
            (eid.id_lo() as i64, eid.id_hi() as i64),
            |row| {
                Ok((
                    row.get::<&str, bool>("is_to")?,
                    row.get::<&str, i64>("r_id_lo")?,
                    row.get::<&str, i64>("r_id_hi")?,
                ))
            },
        );
        if let Err(err) = rows {
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut from = Vec::new();
        let mut to = Vec::new();
        for row in rows.unwrap() {
            match row {
                Ok((is_to, r_id_lo, r_id_hi)) => {
                    let r_eid = EntityId::new(r_id_lo as u64, r_id_hi as u64);
                    if is_to {
                        to.push(r_eid);
                    } else {
                        from.push(r_eid);
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
            ta: self.tags_and_attrs_get(eid)?,
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
            None => self.eid_stored_max()?.unwrap_or(ENTITY_ID_START),
            Some(eid) => *eid,
        };
        self.tags_and_attrs_put(&eid, &link.ta)?;
        let sql = "INSERT INTO links(id_lo, id_hi, \
                                     is_to, r_id_lo, r_id_hi) \
                   VALUES(?, ?, ?, ?, ?);";
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
            for r_eid in vec_eid {
                let nr_inserted = statement.execute((
                    eid.id_lo() as i64,
                    eid.id_hi() as i64,
                    is_to,
                    r_eid.id_lo() as i64,
                    r_eid.id_hi() as i64,
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
        self.tags_and_attrs_del(eid)?;
        let sql = "DELETE FROM links WHERE \
                   id_lo = ? AND id_hi = ?\
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
        let nr_deleted =
            statement.execute((eid.id_lo() as i64, eid.id_hi() as i64));
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
        let sql = "SELECT DISTINCT id_hi, id_lo \
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
        let rows = statement.query_map((), |row| {
            let id_lo = row.get::<&str, i64>("id_lo");
            let id_hi = row.get::<&str, i64>("id_hi");
            Ok((id_lo, id_hi))
        });
        if let Err(err) = rows {
            debug!("err={err}");
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let mut vids = Vec::<EntityId>::new();
        for row in rows.unwrap() {
            match row {
                Ok((id_lo, id_hi)) => {
                    if let Err(err) = id_lo {
                        return Err(Box::new(
                            ContainerSqliteError::SqliteQueryMapFailed { err },
                        ));
                    }
                    let id_lo = id_lo.unwrap() as u64;
                    if let Err(err) = id_hi {
                        return Err(Box::new(
                            ContainerSqliteError::SqliteQueryMapFailed { err },
                        ));
                    }
                    let id_hi = id_hi.unwrap() as u64;
                    vids.push(EntityId::new(id_lo, id_hi));
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
        assert_eq!(tags1, tags);

        tx.tags_del(&eid).unwrap();

        let tags1 = tx.tags_get(&eid).unwrap();
        assert!(tags1.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();
    }

    #[test]
    fn smoke_attrs() {
        crate::app::init();

        let mut container = ContainerSqlite::new("").unwrap();
        container.create().unwrap();
        let mut test_rng = helpers::TestRng::new(1);
        let eid = helpers::random_entity_id(&mut test_rng);
        let mut tx = container.begin_transaction().unwrap();

        let attrs1 = tx.attrs_get(&eid).unwrap();
        assert!(attrs1.is_empty());

        let attrs = helpers::random_attrs(&mut test_rng);
        tx.attrs_put(&eid, &attrs).unwrap();

        let attrs1 = tx.attrs_get(&eid).unwrap();
        assert_eq!(attrs1, attrs);

        tx.attrs_del(&eid).unwrap();

        let attrs1 = tx.attrs_get(&eid).unwrap();
        assert!(attrs1.is_empty());

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
        assert_eq!(record1, record);

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

        let r_eid1 = helpers::random_entity_id(&mut test_rng);
        let record1 = helpers::random_record(&mut test_rng);
        let _ = tx.record_put(&Some(r_eid1), &record1).unwrap();
        let r_eid2 = helpers::random_entity_id(&mut test_rng);
        let record2 = helpers::random_record(&mut test_rng);
        let _ = tx.record_put(&Some(r_eid2), &record2).unwrap();

        let eid = helpers::random_entity_id(&mut test_rng);

        let all_ids = tx.link_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let link = Link {
            ta: helpers::random_tags_and_attrs(&mut test_rng),
            from: vec![r_eid1],
            to: vec![r_eid2],
        };

        let eid1 = tx.link_put(&Some(eid), &link).unwrap();
        assert_eq!(eid1, eid);

        let link1 = tx.link_get(&eid).unwrap().unwrap();
        assert_eq!(link1, link);

        let all_ids = tx.link_get_all_ids().unwrap();
        assert_eq!(all_ids.len(), 1, "all_ids={:?}", all_ids);
        assert_eq!(all_ids[0], eid);

        let deleted = tx.link_del(&eid).unwrap();
        assert!(deleted);
        let all_ids = tx.link_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let deleted = tx.record_del(&r_eid1).unwrap();
        assert!(deleted);
        let deleted = tx.record_del(&r_eid2).unwrap();
        assert!(deleted);
        let r_all_ids = tx.record_get_all_ids().unwrap();
        assert!(r_all_ids.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();
    }
}
