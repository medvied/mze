..
   Copyright 2021 Maksym Medvied

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.


==================
Max's Zettelkasten
==================

Overview
--------

There are different ways to store information. There are multiple ways to store
links to information. This software allows to link pieces of information in the
way that relevant information is easy and fast to find.

Goals:

- adding records/links between records is easy and fast;
- search feature allows to find relevant record as fast as it's technically
  possible. Software MUST NOT be a bottleneck during search.

Glossary
--------

.. glossary::

Record
  Piece of information.

Link
  An 1:1 relation between records.


Features
--------

- search: full-text, OCRed images, regexp, search over multiple MZEs (federated
  search)
- version history: like git commit message for every change
- migration: partial, by record type, select what to migrate, copy or move
- API: everything within MZE is available for scripts
- plugins: extend functionality


Requirements
------------

.. list-table::

    * - R.uuid
      - Each record and each link have their own UUID.
    * - R.git
      - Each record and link are stored in git.
    * - R.record.image
      - Image records are supported.
    * - R.record.sound
      - Sound files are records are supported.
    * - R.record.video
      - Video records are supported.
    * - R.reference.time
      - Time reference should be possible.
    * - R.reference.place
      - Place reference should be possible. GPS coordinates, country, etc.
    * - R.record.blob
      - Blobs should be supported.
    * - R.record.reference-counter
      - Records MUST have reference counters.
    * - R.record.retire-policy
      - It should be a retire policy on when to remove a record. Examples: no
        references + 1y etc.
    * - R.record.remove-deferred
      - All removals should be deferred with configurable timeout.
    * - R.fuzzy-search
      - Fuzzy search feature should implemented. Example: github -> go to file.
    * - R.record.mime-type
      - Each record has to have it's MIME type.
    * - R.record.uri
      - Each record has to have it's own URI
    * - R.integration
      - It should be possible to have integrations
    * - R.integration.torrent
      - It should be possible to integrate with torrent clients. Example: add a
        .torrent file or a magnet link, have a button to download the torrent.
        Button press initiates download and then updates link to the data. Also
        retiring policies. Use case: watching movies/series.
    * - R.integration.web
      - It should be possible to download web pages references by links.
    * - R.distributed
      - It should be possible to modify MZE from any place. All modifications
        should be in-sync.
    * - R.outliner
      - Outliners MUST be supported. Outliner is a special kind of record that
        has a tree-like structure with links to other records.
    * - R.integration.scanner
      - Scanner should add a record directly to MZE.
    * - R.integration.photo.mobile
      - Photos from mobile phone should be added to MZE.
    * - R.integration.camera
      - Photos from camera should be added to MZE.
    * - R.record.version
      - Record versioning MUST be supported. It MUST be possible to see every
        version of every record.
    * - R.version-global
      - MZE MUST have a global version. It MUST be possible to choose any
        global version.
    * - R.record.tombstone
      - Removed records MUST have tombstones, that describe what happened, when
        and why.
    * - R.backup
      - It MUST be possible to backup full MZE.
    * - R.backup.incremental
      - It MUST be possible to make incremental MZE backup.
    * - R.tag
      - It MUST be possible to add a list of tags for every record.
    * - R.check-if-needed.tag-tree
      - It MUST be possible to add a tree-like structure of tags. It would
        allow to narrow the search.
    * - R.record.date
      - It MUST be possible to add different kind of dates (see ``stat``) to
        each record.
    * - R.partial-export
      - It MUST be possible to export a part of MZE to another MZE. Use case:
        export public data, export technical sub-MZE.
    * - R.import
      - It MUST be possible to merge 2 MZEs.
    * - R.attribute
      - Each record MAY have attributes defined. They include: location, time,
        ...
    * - R.query
      - It MUST be possible to query records based on tags, attributes or
        content.
    * - R.query.fuzzy
      - Fuzzy queries must be supported.
    * - R.query.regexp
      - Regular expression query MUST be supported.
    * - R.update-feed
      - Atom/RSS feeds MUST be available about updates to each record.
    * - R.record.query
      - A query kind of record MUST be supported. It content is dynamic and is
        based on current MZE state. Example: "All algorithms".
    * - R.link.embed
      - Link MUST have "embed" type. Example: an image is embedded into the
        text.
    * - R.record.view
      - It MUST be possible to have a view kind of record. View could show
        existing data, but in different format. Example: previews of the
        images, mount database as a filesystem etc.
    * - R.record.executable
      - It MUST be possible to have an executable record. Executable record
        allows to do something somewhere when it's "executed". Includes: spawn
        a process on a host etc.
    * - R.record.timer
      - A timer that may trigger another executable record.
    * - R.plugin
      - Plugins MUST be supported. Storage plugin, URI plugin etc.
    * - R.record.table
      - Table kind of record. Resembles SQL database table.
    * - R.records.10M
      - 10M records MUST be supported.
    * - R.rendering.instant
      - Modifications should be re-rendered in the open windows immediately.
    * - R.record.sequence
      - Record type - sequence. Allows to create an ordered list of records.
    * - R.link.type
      - It MUST be possible to set link type.
    * - R.record.templace
      - It MUST be possible to have templates that new records may be based on.
        Template record provides a way to automatically parse records based on
        this template.
    * - R.future-proof
      - The solution MUST be future-proof. It MUST be possible to use it in 50
        years, regardless of life time of the software the solution is based
        on.
    * - R.record.feed
      - It MUST be possible to have pointers to RSS/Atom, internal
        (R.update-feed) or external. It MUST be possible to aggregate several
        such records into a single record. It MUST be possible to specify
        update interval etc.
    * - R.export.git
      - Export to a git repo with version history and changes like they were
        done to a single record (i.e. without having all versions present in
        the last commit)


Design
------

- mze-rs - MZE record server
- mze-re - MZE renderer
- mze-ss - MZE storage server
- mze-ca - MZE cache
- mze-sm - MZE system manager
- mze-pr - MZE proxy
- mze-se - MZE search engine


Technologies
------------

- Docker to run everything in containers
- Python as the primary language
- Web browser as UI
- http(s) as mze-re <-> browser transport
- MZE protocol to retrive records
- FUSE to access remote records as files when needed (use case: large records
- like movies)
- files on a filesystem for everything
- S3 for blobs (?)
- Records UUID to records location mapping
- Record UUID to tags mapping
- Record UUID to attribute mapping
- Record UUID, tags, attributes, URIs - metadata
- Git for metadata versioning. Rewrite git history as needed.
- nginx as web server/proxy
- neovim as the editor

Directory structure:
- first 2 digits of UUID
- second 2 digits of UUID
- full UUID

Alternative:
- 0
- 1
- ...
- 999
- 1000/1000
- 1000/1001
- ...
- 1000/1999
- 2000/2000
- ...

File structure for an record
- ``tags``: json list of tags
- ``attributes``: json map of attributes
- ``uri``: record URI. May be the same for different records.

``versions`` dir. Has dirs, name = number. To create a version all files from
previous version are moved to the version dir. Version history is derived from
``git log``. Object changes are tied together with git commits.

- mze-rs is a RESTful server that manages git repo

  - request: record UUIDs and what to do with them.

- mze-ss gives records by URIs.

  - request: URI GET/PUT
  - reply: data or redirect to another mze-ss

- diagram software

  - https://gojs.net/latest/samples/sequenceDiagram.html
  - https://visjs.org/
  - https://d3js.org/
  - https://mermaid-js.github.io/mermaid/#/


MVP
---

- nginx as a web server
- git for metadata
- filesystem for records
- rst file format
- rst2html5 renderer
- vim to modify


Existing implementations
------------------------

Articles
........

- https://en.wikipedia.org/wiki/Personal_knowledge_base
- https://en.wikipedia.org/wiki/Personal_knowledge_management
- https://zettelkasten.de/posts/overview/
- https://notes.andymatuschak.org/About_these_notes
- https://notes.andymatuschak.org/z3SjnvsB5aR2ddsycyXofbYR7fCxo7RmKW2be
- `Trilium Notes is a hierarchical note taking application with focus on building large personal knowledge bases <https://github.com/zadam/trilium>`_
- `Как я веду Zettelkasten в Notion уже год: стартовый набор и полезные трюки <https://habr.com/ru/post/509756/>`_
- https://dangirsh.org/posts/zettelkasten.html
- https://en.wikipedia.org/wiki/User_modeling
- https://en.wikipedia.org/wiki/Personal_wiki
- https://en.wikipedia.org/wiki/Information_mapping
- https://en.wikipedia.org/wiki/Mind_map
- https://orgmode.org/
- https://en.wikipedia.org/wiki/Comparison_of_note-taking_software
- https://en.wikipedia.org/wiki/Comparison_of_document-markup_languages
- https://en.wikipedia.org/wiki/List_of_personal_information_managers
- https://en.wikipedia.org/wiki/Outliner
- https://en.wikipedia.org/wiki/Comparison_of_note-taking_software


Alternatives
............

- https://ru.wikipedia.org/wiki/MyTetra
- https://en.wikipedia.org/wiki/TagSpaces
- https://en.wikipedia.org/wiki/Taskwarrior
- https://en.wikipedia.org/wiki/TiddlyWiki
- https://en.wikipedia.org/wiki/Leo_(text_editor)
- https://en.wikipedia.org/wiki/Tomboy_(software)
- https://en.wikipedia.org/wiki/QOwnNotes
- https://en.wikipedia.org/wiki/MyNotex
- https://en.wikipedia.org/wiki/BasKet_Note_Pads
- https://en.wikipedia.org/wiki/Gnote

.. list-table::

    * - name
      - features
      - what is missing
    * - `Org Mode <https://orgmode.org/>`_ (`source
        <https://code.orgmode.org/bzg/org-mode>`_ `wiki
        <https://en.wikipedia.org/wiki/Org-mode>`_)
      -
      -
    * - `Zim <https://zim-wiki.org/>`_ is a graphical text editor used to
        maintain a collection of wiki pages  (`source
        <https://github.com/zim-desktop-wiki/zim-desktop-wiki>`_ `wiki
        <https://en.wikipedia.org/wiki/Zim_(software)>`_)
      -
      -
