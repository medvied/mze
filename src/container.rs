use std::error;

use crate::Container;

pub mod ram;
pub mod sqlite;

pub fn new(
    name: &str,
    uri: &str,
) -> Result<Box<dyn Container + Send>, Box<dyn error::Error>> {
    Ok(match name {
        "sqlite" => Box::new(sqlite::ContainerSqlite::new(uri)?),
        "ram" => Box::new(ram::ContainerRam::new(uri)?),
        _ => panic!("container::new(): unknown name={name}"),
    })
}
