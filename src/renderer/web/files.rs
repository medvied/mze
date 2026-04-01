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

pub const SEARCH_HTML: &[u8] = include_bytes!("search.html");
pub const SEARCH_CSS: &[u8] = include_bytes!("search.css");
pub const SEARCH_JS: &[u8] = include_bytes!("search.js");

pub const TEMPLATES: &[(&str, &[u8])] = &[
    (
        "search_result_tag.html",
        include_bytes!("search_result_tag.html"),
    ),
    (
        "search_result_attribute.html",
        include_bytes!("search_result_attribute.html"),
    ),
    (
        "search_result_record.html",
        include_bytes!("search_result_record.html"),
    ),
    (
        "search_result_link.html",
        include_bytes!("search_result_link.html"),
    ),
];
