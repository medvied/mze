use std::{
    collections::{HashMap, HashSet},
    error,
};

use log::{debug, error};

use rusqlite;

use thiserror::Error;

use crate::{
    Container, ContainerTransaction, EntityId, EntityIdVer, Link, Record,
    SearchResult,
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
    #[error("too many rows for a single record: eidv={eidv:?}")]
    TooManyRowsForARecord { eidv: EntityIdVer },
    #[error("error retrieving record data: err={err}")]
    ErrorRetrievingRecordData { err: rusqlite::Error },
    #[error("error executing prepared statement: sql={sql} err={err}")]
    ErrorExecutingStatement { sql: String, err: rusqlite::Error },
    #[error("failed to insert 1 entry: sql={sql} inserted={nr_inserted}")]
    FailedToInsert1Entry { sql: String, nr_inserted: usize },
    #[error("failed to get record max version: sql={sql} err={err}")]
    FailedToGetRecordMaxVer { sql: String, err: rusqlite::Error },
    #[error("duplicate tags in container: eidv={eidv:?} tag={tag}")]
    DuplicateTagsInContainer { eidv: EntityIdVer, tag: String },
    #[error(
        "duplicate attrs in container: eidv={eidv:?} key={key} value={value}"
    )]
    DuplicateAttrsInContainer {
        eidv: EntityIdVer,
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
    pub fn new(uri: &str) -> Result<ContainerSqlite, ContainerSqliteError> {
        let conn = if uri.is_empty() {
            rusqlite::Connection::open_in_memory()
        } else {
            rusqlite::Connection::open(uri)
        };
        match conn {
            Ok(conn) => Ok(ContainerSqlite { conn }),
            Err(e) => Err(ContainerSqliteError::CantOpenSqliteConnection {
                uri: uri.to_string(),
                err: e,
            }),
        }
    }

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
    /// TODO use NOT NULL here and check for NULL
    fn create(&self) -> Result<(), Box<dyn error::Error>> {
        let statements: &[&str] = &[
            "CREATE TABLE records(\
                id_lo INTEGER, \
                id_hi INTEGER, \
                ver INTEGER, \
                data BLOB\
            ) STRICT;",
            "CREATE TABLE tags(\
                id_lo INTEGER, \
                id_hi INTEGER, \
                ver INTEGER, \
                tag TEXT\
            ) STRICT;",
            "CREATE TABLE attrs(\
                id_lo INTEGER, \
                id_hi INTEGER, \
                ver INTEGER, \
                key TEXT, \
                value TEXT\
            ) STRICT;",
            "CREATE TABLE links(\
                id_lo INTEGER, \
                id_hi INTEGER, \
                ver INTEGER, \
                is_to INTEGER, \
                r_id_lo INTEGER, \
                r_id_hi INTEGER, \
                r_ver INTEGER\
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
        eidv: &EntityIdVer,
    ) -> Result<HashSet<String>, Box<dyn error::Error>> {
        let sql = "SELECT tag \
             FROM tags \
             WHERE \
             id_lo = ? AND id_hi = ? AND ver = ?\
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
            (eidv.id_lo() as i64, eidv.id_hi() as i64, eidv.ver() as i64),
            |row| row.get::<&str, String>("tag"),
        );
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
                                eidv: eidv.clone(),
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
        eidv: &EntityIdVer,
        tags: &HashSet<String>,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "INSERT INTO tags(id_lo, id_hi, ver, tag) \
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
        for tag in tags {
            let nr_inserted = statement.execute((
                eidv.id_lo() as i64,
                eidv.id_hi() as i64,
                eidv.ver() as i64,
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
        eidv: &EntityIdVer,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "DELETE FROM tags WHERE \
                   id_lo = ? AND id_hi = ? AND ver = ?\
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
        let nr_deleted = statement.execute((
            eidv.id_lo() as i64,
            eidv.id_hi() as i64,
            eidv.ver() as i64,
        ));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        debug!(
            "tags_del(): eidv={eidv:?} nr_deleted={}",
            nr_deleted.unwrap()
        );
        Ok(())
    }

    fn attrs_get(
        &self,
        eidv: &EntityIdVer,
    ) -> Result<HashMap<String, String>, Box<dyn error::Error>> {
        let sql = "SELECT key, value \
             FROM attrs \
             WHERE \
             id_lo = ? AND id_hi = ? AND ver = ?\
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
            (eidv.id_lo() as i64, eidv.id_hi() as i64, eidv.ver() as i64),
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
                                eidv: eidv.clone(),
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
        eidv: &EntityIdVer,
        attrs: &HashMap<String, String>,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "INSERT INTO attrs(id_lo, id_hi, ver, key, value) \
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
        for (key, value) in attrs {
            let nr_inserted = statement.execute((
                eidv.id_lo() as i64,
                eidv.id_hi() as i64,
                eidv.ver() as i64,
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
        eidv: &EntityIdVer,
    ) -> Result<(), Box<dyn error::Error>> {
        let sql = "DELETE FROM attrs WHERE \
                   id_lo = ? AND id_hi = ? AND ver = ?\
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
        let nr_deleted = statement.execute((
            eidv.id_lo() as i64,
            eidv.id_hi() as i64,
            eidv.ver() as i64,
        ));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        debug!(
            "attrs_del(): eidv={eidv:?} nr_deleted={}",
            nr_deleted.unwrap(),
        );
        Ok(())
    }

    fn record_get(
        &self,
        eidv: &EntityIdVer,
    ) -> Result<Option<Record>, Box<dyn error::Error>> {
        let sql = "SELECT data \
             FROM records \
             WHERE \
             id_lo = ? AND id_hi = ? AND ver = ?\
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
            (eidv.id_lo() as i64, eidv.id_hi() as i64, eidv.ver() as i64),
            |row| row.get::<&str, Vec<u8>>("data"),
        );
        if let Err(err) = rows {
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let rows: Vec<_> = rows.unwrap().collect();
        if rows.len() > 1 {
            return Err(Box::new(
                ContainerSqliteError::TooManyRowsForARecord {
                    eidv: eidv.clone(),
                },
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
            ta: self.tags_and_attrs_get(eidv)?,
            data: Some(data),
        };
        Ok(Some(record))
    }

    fn record_put(
        &mut self,
        eid: &EntityId,
        record: &Record,
    ) -> Result<EntityIdVer, Box<dyn error::Error>> {
        let eidv = self.record_get_ver_latest(eid)?;
        let eidv = EntityIdVer {
            id: eid.id,
            ver: match eidv {
                Some(eidv) => eidv.ver + 1,
                None => 1,
            },
        };
        self.tags_and_attrs_put(&eidv, &record.ta)?;
        let sql = "INSERT INTO records(id_lo, id_hi, ver, data) \
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
        let nr_inserted = statement.execute((
            eidv.id_lo() as i64,
            eidv.id_hi() as i64,
            eidv.ver() as i64,
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
        Ok(eidv)
    }

    fn record_del(
        &mut self,
        eidv: &EntityIdVer,
    ) -> Result<bool, Box<dyn error::Error>> {
        self.tags_and_attrs_del(eidv)?;
        let sql = "DELETE FROM records WHERE \
                   id_lo = ? AND id_hi = ? AND ver = ?\
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
        let nr_deleted = statement.execute((
            eidv.id_lo() as i64,
            eidv.id_hi() as i64,
            eidv.ver() as i64,
        ));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
        Ok(nr_deleted.unwrap() > 0)
    }

    fn record_get_all_ids(
        &self,
    ) -> Result<Vec<EntityId>, Box<dyn error::Error>> {
        let sql = "SELECT DISTINCT id_hi, id_lo \
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
        Ok(vids)
    }

    fn record_get_ver_latest(
        &self,
        eid: &EntityId,
    ) -> Result<Option<EntityIdVer>, Box<dyn error::Error>> {
        let sql = "SELECT max(ver) \
             FROM records \
             WHERE \
             id_lo = ? AND id_hi = ?\
             ;";
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
        let rows = statement.query_map(
            (eid.id_lo() as i64, eid.id_hi() as i64),
            |row| {
                let value = row.get_ref(0);
                match value {
                    Ok(v) => match Into::<rusqlite::types::Value>::into(v) {
                        rusqlite::types::Value::Null => Ok(None),
                        rusqlite::types::Value::Integer(i) => Ok(Some(i)),
                        _ => panic!(),
                    },
                    Err(err) => Err(err),
                }
            },
        );
        if let Err(err) = rows {
            debug!("err={err}");
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let rows: Vec<_> = rows.unwrap().collect();
        assert!(rows.len() <= 1);
        if rows.is_empty() {
            Ok(None)
        } else {
            let first = rows.into_iter().nth(0);
            match first {
                Some(row) => match row {
                    Ok(v) => match v {
                        Some(ver) => Ok(Some(EntityIdVer {
                            id: eid.id,
                            ver: ver as u64,
                        })),
                        None => Ok(None),
                    },
                    Err(err) => Err(Box::new(
                        ContainerSqliteError::FailedToGetRecordMaxVer {
                            sql: sql.to_string(),
                            err,
                        },
                    )),
                },
                None => panic!(),
            }
        }
    }

    fn link_get(
        &self,
        eidv: &EntityIdVer,
    ) -> Result<Option<Link>, Box<dyn error::Error>> {
        let sql = "SELECT is_to, r_id_lo, r_id_hi, r_ver \
             FROM links \
             WHERE \
             id_lo = ? AND id_hi = ? AND ver = ?\
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
            (eidv.id_lo() as i64, eidv.id_hi() as i64, eidv.ver() as i64),
            |row| {
                Ok((
                    row.get::<&str, bool>("is_to")?,
                    row.get::<&str, i64>("r_id_lo")?,
                    row.get::<&str, i64>("r_id_hi")?,
                    row.get::<&str, i64>("r_ver")?,
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
                Ok((is_to, r_id_lo, r_id_hi, r_ver)) => {
                    let r_eidv = EntityIdVer::new(
                        r_id_lo as u64,
                        r_id_hi as u64,
                        r_ver as u64,
                    );
                    if is_to {
                        to.push(r_eidv);
                    } else {
                        from.push(r_eidv);
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
            ta: self.tags_and_attrs_get(eidv)?,
            from,
            to,
        };
        Ok(Some(link))
    }

    fn link_put(
        &mut self,
        eid: &EntityId,
        link: &Link,
    ) -> Result<EntityIdVer, Box<dyn error::Error>> {
        let eidv = self.link_get_ver_latest(eid)?;
        let eidv = EntityIdVer {
            id: eid.id,
            ver: match eidv {
                Some(eidv) => eidv.ver + 1,
                None => 1,
            },
        };
        self.tags_and_attrs_put(&eidv, &link.ta)?;
        let sql = "INSERT INTO links(id_lo, id_hi, ver, \
                                     is_to, r_id_lo, r_id_hi, r_ver) \
                   VALUES(?, ?, ?, ?, ?, ?, ?);";
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
        for (is_to, vec_eidv) in [(false, &link.from), (true, &link.to)] {
            for r_eidv in vec_eidv {
                let nr_inserted = statement.execute((
                    eidv.id_lo() as i64,
                    eidv.id_hi() as i64,
                    eidv.ver() as i64,
                    is_to,
                    r_eidv.id_lo() as i64,
                    r_eidv.id_hi() as i64,
                    r_eidv.ver() as i64,
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
        Ok(eidv)
    }

    fn link_del(
        &mut self,
        eidv: &EntityIdVer,
    ) -> Result<bool, Box<dyn error::Error>> {
        self.tags_and_attrs_del(eidv)?;
        let sql = "DELETE FROM links WHERE \
                   id_lo = ? AND id_hi = ? AND ver = ?\
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
        let nr_deleted = statement.execute((
            eidv.id_lo() as i64,
            eidv.id_hi() as i64,
            eidv.ver() as i64,
        ));
        if let Err(err) = nr_deleted {
            return Err(Box::new(
                ContainerSqliteError::ErrorExecutingStatement {
                    sql: sql.to_string(),
                    err,
                },
            ));
        }
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
        Ok(vids)
    }

    fn link_get_ver_latest(
        &self,
        eid: &EntityId,
    ) -> Result<Option<EntityIdVer>, Box<dyn error::Error>> {
        let sql = "SELECT max(ver) \
             FROM links \
             WHERE \
             id_lo = ? AND id_hi = ?\
             ;";
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
        let rows = statement.query_map(
            (eid.id_lo() as i64, eid.id_hi() as i64),
            |row| {
                let value = row.get_ref(0);
                match value {
                    Ok(v) => match Into::<rusqlite::types::Value>::into(v) {
                        rusqlite::types::Value::Null => Ok(None),
                        rusqlite::types::Value::Integer(i) => Ok(Some(i)),
                        _ => panic!(),
                    },
                    Err(err) => Err(err),
                }
            },
        );
        if let Err(err) = rows {
            debug!("err={err}");
            return Err(Box::new(
                ContainerSqliteError::SqliteQueryMapFailed { err },
            ));
        }
        let rows: Vec<_> = rows.unwrap().collect();
        assert!(rows.len() <= 1);
        if rows.is_empty() {
            Ok(None)
        } else {
            let first = rows.into_iter().nth(0);
            match first {
                Some(row) => match row {
                    Ok(v) => match v {
                        Some(ver) => Ok(Some(EntityIdVer {
                            id: eid.id,
                            ver: ver as u64,
                        })),
                        None => Ok(None),
                    },
                    Err(err) => Err(Box::new(
                        ContainerSqliteError::FailedToGetRecordMaxVer {
                            sql: sql.to_string(),
                            err,
                        },
                    )),
                },
                None => panic!(),
            }
        }
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
        let eidv = helpers::random_entity_id_ver(&mut test_rng);
        let mut tx = container.begin_transaction().unwrap();

        let tags1 = tx.tags_get(&eidv).unwrap();
        assert!(tags1.is_empty());

        let tags = helpers::random_tags(&mut test_rng);
        tx.tags_put(&eidv, &tags).unwrap();

        let tags1 = tx.tags_get(&eidv).unwrap();
        assert_eq!(tags1, tags);

        tx.tags_del(&eidv).unwrap();

        let tags1 = tx.tags_get(&eidv).unwrap();
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
        let eidv = helpers::random_entity_id_ver(&mut test_rng);
        let mut tx = container.begin_transaction().unwrap();

        let attrs1 = tx.attrs_get(&eidv).unwrap();
        assert!(attrs1.is_empty());

        let attrs = helpers::random_attrs(&mut test_rng);
        tx.attrs_put(&eidv, &attrs).unwrap();

        let attrs1 = tx.attrs_get(&eidv).unwrap();
        assert_eq!(attrs1, attrs);

        tx.attrs_del(&eidv).unwrap();

        let attrs1 = tx.attrs_get(&eidv).unwrap();
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

        let eidv = tx.record_get_ver_latest(&eid).unwrap();
        assert!(eidv.is_none());

        let all_ids = tx.record_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let record = helpers::random_record(&mut test_rng);
        let eidv = tx.record_put(&eid, &record).unwrap();
        assert_eq!(eid.id, eidv.id);
        assert_eq!(eidv.ver, 1);

        let record1 = tx.record_get(&eidv).unwrap().unwrap();
        assert_eq!(record1, record);

        let eidv1 = tx.record_get_ver_latest(&eid).unwrap().unwrap();
        assert_eq!(eidv1, eidv);

        let all_ids = tx.record_get_all_ids().unwrap();
        assert_eq!(all_ids.len(), 1);
        assert_eq!(all_ids[0], eid);

        let deleted = tx.record_del(&eidv).unwrap();
        assert!(deleted);

        let record1 = tx.record_get(&eidv).unwrap();
        assert!(record1.is_none());

        let eidv = tx.record_get_ver_latest(&eid).unwrap();
        assert!(eidv.is_none());

        let all_ids = tx.record_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();
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
        let r_eidv1 = tx.record_put(&r_eid1, &record1).unwrap();
        let r_eid2 = helpers::random_entity_id(&mut test_rng);
        let record2 = helpers::random_record(&mut test_rng);
        let r_eidv2 = tx.record_put(&r_eid2, &record2).unwrap();

        let eid = helpers::random_entity_id(&mut test_rng);

        let eidv = tx.link_get_ver_latest(&eid).unwrap();
        assert!(eidv.is_none());
        let all_ids = tx.link_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let link = Link {
            ta: helpers::random_tags_and_attrs(&mut test_rng),
            from: vec![r_eidv1.clone()],
            to: vec![r_eidv2.clone()],
        };

        let eidv = tx.link_put(&eid, &link).unwrap();
        assert_eq!(eid.id, eidv.id);
        assert_eq!(eidv.ver, 1);

        let link1 = tx.link_get(&eidv).unwrap().unwrap();
        assert_eq!(link1, link);

        let eidv1 = tx.link_get_ver_latest(&eid).unwrap().unwrap();
        assert_eq!(eidv1, eidv);
        let all_ids = tx.link_get_all_ids().unwrap();
        assert_eq!(all_ids.len(), 1);
        assert_eq!(all_ids[0], eid);

        let deleted = tx.link_del(&eidv).unwrap();
        assert!(deleted);
        let eidv = tx.link_get_ver_latest(&eid).unwrap();
        assert!(eidv.is_none());
        let all_ids = tx.link_get_all_ids().unwrap();
        assert!(all_ids.is_empty());

        let deleted = tx.record_del(&r_eidv1).unwrap();
        assert!(deleted);
        let deleted = tx.record_del(&r_eidv2).unwrap();
        assert!(deleted);
        let r_all_ids = tx.record_get_all_ids().unwrap();
        assert!(r_all_ids.is_empty());

        tx.commit().unwrap();
        container.destroy().unwrap();
    }
}
