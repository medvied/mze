// SPDX-License-Identifier: AGPL-3.0-or-later
/*
    mze - personal knowledge database
    Copyright (C) 2024, 2025  Maksym Medvied

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
