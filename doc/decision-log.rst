..
    SPDX-License-Identifier: AGPL-3.0-or-later

    mze - personal knowledge database
    Copyright (C) 2021, 2025  Maksym Medvied

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


No versioning on C.storage level
--------------------------------

- pros

  - easier C.storage implementation

- cons

  - more complex C.renderer implementation


Use S3 protocol instead of custom S3-like protocol
--------------------------------------------------

- references

  - R.glue
  - R.uuid
  - R.record.uri
  - C.storage-server
  - C.kvdb

- pros

  - there are many tools out there that are already there that are compatible
    with S3 protocol
  - no need to invent and debug a new protocol
  - the data storing part in S3 protocol is as easy as whatever is implemented
    now

- neutral

  - a need to implement extensions to the protocol may limit the ability to use
    existing S3 implementations as MZE storage server

- cons

  - there are things in S3 protocol that are not needed for MZE, like
    buckets or authentication
  - protocol extensions may be needed
  - limitations would be there in MZE implementation which may reduce the
    amount of S3 tools that could use MZE implementation

- ideas

  - implement S3 in addition to the internal protocol. This would allow to
    inspect the data through S3 and use internal protocol internally.


No versioning on mze level
--------------------------

- pros

  - much easier to implement
  - no ambiguity on which record version to show and how to handle search when
    multiple records match

- cons

  - some kind of lossy or lossless versioning is still needed outside of mze
    (example: backups)
  - no way to search the old versions - if the data was there it would not be
    found


Use u64 entity id instead of u128
---------------------------------

- rationale

  - u64 is enough to fit any personal knowledge database for the foreseeable
    future
  - if a need arises there is also u64 container namespace

- pros

  - easier to implement
  - easier to display
  - easier to link to from other records in text (like mze:///record_id)

- cons

  - in some cases it may be harder to split bits of the entity id when many
    categories of records/links are needed and they are encoded in the parts of
    the entity ids
