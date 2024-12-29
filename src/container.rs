use std::error;

use crate::Container;

pub mod sqlite;

pub fn new(
    name: &str,
    uri: &str,
) -> Result<Box<dyn Container>, Box<dyn error::Error>> {
    Ok(Box::new(match name {
        "sqlite" => sqlite::ContainerSqlite::new(uri)?,
        _ => panic!("container::new(): unknown name={name}"),
    }))
}
