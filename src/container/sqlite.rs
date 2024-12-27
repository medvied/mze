use std::{
    collections::{HashMap, HashSet},
    error,
};

use log::{debug, error};

use rusqlite;

use thiserror::Error;

use crate::{
    Container,
    ContainerTransaction,
    EntityIdVer,
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
    type Transaction<'a> = ContainerSqliteTransaction<'a>;

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
        ];
        self.statements_execute(statements)
    }

    fn destroy(&self) -> Result<(), Box<dyn error::Error>> {
        let statements: &[&str] = &[
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
    ) -> Result<Self::Transaction<'_>, Box<dyn error::Error>> {
        let tx = self.conn.transaction();
        match tx {
            Ok(tx) => Ok(ContainerSqliteTransaction { tx }),
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
    fn commit(self) -> Result<(), Box<dyn error::Error>> {
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

    fn rollback(self) -> Result<(), Box<dyn error::Error>> {
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
}