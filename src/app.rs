// SPDX-License-Identifier: AGPL-3.0-or-later
/*
    mze - personal knowledge database
    Copyright (C) 2024  Maksym Medvied

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

use std::{io::Write, sync, thread};

use color_backtrace;
use env_logger;

pub fn init() {
    static ONCE: sync::Once = sync::Once::new();

    ONCE.call_once(|| {
        color_backtrace::install();

        env_logger::Builder::from_default_env()
            .format(|buf, record| {
                let style = buf.default_level_style(record.level());
                writeln!(
                    buf,
                    "{} {:>3} {}:{:>03} {} {style}{}{style:#} {}",
                    buf.timestamp_nanos(),
                    thread::current().id().as_u64(),
                    record.file().unwrap(),
                    record.line().unwrap(),
                    thread::current().name().unwrap_or("UNNAMED"),
                    record.level(),
                    record.args()
                )
            })
            .init();
    });
}
