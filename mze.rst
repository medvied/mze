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


====================
Medvied Zettelkasten
====================

Overview
========

There are different ways to store information. There are multiple ways to store
links to information. This software allows to link pieces of information in the
way that relevant information is easy and fast to find.

Goals:

- adding records/links between records is easy and fast;
- search feature allows to find relevant record as fast as it's technically
  possible. Software MUST NOT be a bottleneck during search.

Glossary
========

.. glossary::

Record
  Piece of information.

Link
  An 1:1 relation between records.


Features
========

- search: full-text, OCRed images, regexp, search over multiple MZEs (federated
  search)
- version history: like git commit message for every change
- migration: partial, by record type, select what to migrate, copy or move
- API: everything within MZE is available for scripts
- plugins: extend functionality


Requirements
============

.. list-table::

    * - R.glue
      - Glue existing projects together to achieve the functionality.
    * - R.uuid
      - Each record and each link have their own UUID.
    * - R.git
      - Each record and link are stored in git.
    * - R.record.immutable
      - Records that couldn't be changed MUST be supported.
    * - R.record.mutable
      - Recrods that could be changed MUST be supported.
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
    * - R.fuzzy-search.interpretations
      - Fuzzy search MUST show how the search query is in interpreted. Example:
        search query "date:01/02", interpretation: "February 1".
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
    * - R.archive
      - It MUST be possible to archive external references (web pages, images,
        maybe sites etc.), so they would be available even if the URI is no
        longer valid.
    * - R.dead-link-detector
      - It MUST be a way to detect and show all internal and external dead
        links.
    * - R.intregration.git
      - It MUST be possible to set up an update to a record whenever a git repo
        is updated.
    * - R.record.kabnan
      - Kanban board kind of recort MUST be supported
    * - R.record.gantt
      - Gantt chart lind of record MUST be supported
    * - R.record.timeline
      - Timeline kind of record MUST be supported
    * - R.changelog
      - Per-record, per-link and per-MZE changelogs MUST be supported.
    * - R.audit-log
      - Audit log (which also includes read-only access) MUST be supported.
    * - R.cli
      - CLI interface MUST be supported
    * - R.instance
      - MZE instance abstraction MUST be supported. Each component
        implementation that is running somewhere is an instance.
    * - R.subscription
      - It MUST be possible to subscribe on events like record operations
        (create/remove/etc.), appearance of new records with some tags, GET for
        a record etc.
    * - R.view.video-from-description
      - It MUST be possible to define a text (which is rendered as a still
        image) and a sound file and view them as a video with text-as-image as
        video and sound file and sound in the video.


Design
======

================  =====  ======================================================
component         short  description
================  =====  ======================================================
C.storage         C.st   storage client/server: blob & alike storage
C.kvdb            C.db   key-value database
C.view-server     C.vs   view server: transform data to a different form
C.renderer        C.re   renderer: put the record together
C.modifier        C.mo   modifier: a way to add/modify/remove/etc. a record
C.manager         C.ma   manager: HA, startup/shutdown, recovery, update
C.cache           C.ca   cache: volatile storage for records with fast access
C.pxoxy           C.pr   proxy: frontend for user
C.search-engine   C.se   search engine: a way to find records
C.client          C.cl   client: browser, neovim, CLI
C.executor        C.ex   executor: automatic actions

C.storage-server  C.sts  server part for C.st
C.storage-client  C.stc  client part for C.st
C.kvdb-server     C.dbs  server part for C.db
C.kvdb-client     C.dbc  client part for C.db
================  =====  ======================================================

================  ================  ===========================================
kind              component         description
================  ================  ===========================================
C.storage-server  C.sts.git         git
.                 C.sts.s3          AWS S3
C.kvdb
C.view-server
C.renderer        C.re.rst2html5
.                 C.re.pandoc
C.manager
C.cache
C.pxoxy           C.pr.nginx        Nginx
C.search-engine
C.client          C.cl.browser      web browser
.                 C.cl.nvim         Neovim
.                 C.cl.fuse         FUSE
.                 C.cl.caldav       CalDAV
================  ================  ===========================================


When updating python/nginx/etc. versions
----------------------------------------

- copy new ``/etc/nginx/conf.d/default.conf`` file from the image;


URI
---

Scheme::

        mze://instance_UUID/record_or_link_UUID/version_UUID?k=v&k1=v1#fragment
        ^                                                     ^
        protocol                                           query


Metadata
--------

- UUID
- version UUID
- URI
- tags
- attributes


MZE attributes
--------------

- MZE attributes start from 'mze.'
- common

  - mze.kind = record | link
  - mze.name = human_readable_name_of_the_record_like_filename

- record

  - mze.data = URI

- link

  - mze.from = URI
  - mze.to = URI
  - mze.directed = bool


Technologies
============

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
===

First
-----

================  ===================  ========================================
role              name                 description
================  ===================  ========================================
C.storage-server  C.sts.dir            - a directory is served directly over
                                         HTTP
                                       - list of files is a special file
C.kvdb            C.dbs.git-ssh        openssh + mounted git repo
C.view-server     C.vs.pdf-page        - input: pdf file + page #
                                       - output: image of the file
                                       - no persistence
C.renderer        C.re.search          search page with results
C.modifier        C.mo.none
C.manager         C.ma.docker-compose  - single docker-compose yaml
C.cache           C.ca.mem             - in-memory cache for records
C.pxoxy           C.pr.nginx
C.pxoxy           C.pr.all-records     returns record body by record uuid
C.search-engine   C.se.pdf-pages       - input: string
                                       - output: list[pdf file, page #,
                                         image around]
C.client          C.cl.firefox
C.executor        C.ex.none
================  ===================  ========================================

Interaction diagram
...................

- ``C.cl.firefox`` -> ``C.pr.nginx`` -> (``C.pr.all-records``, ``C.re.search``)
- ``C.pr.all-records`` -> ``C.ca.mem``
- ``C.re.search`` -> (``C.ca.mem``, ``C.se.pdf-pages``)
- ``C.ca.mem`` -> (``C.vs.pdf-page``, ``C.sts.dir``, ``C.dbs.git-ssh``)
- ``C.vs.pdf-page`` -> ``C.sts.dir``

Implementation plan
...................

- ``C.pr.nginx``: reverse proxy for ``C.pr.all-records``, ``C.re.search`` and
  for debugging: ``C.sts.dir``, ``C.vs.pdf-page``, ``C.ca.mem``,
  ``C.se.pdf-page``.
- ``C.sts.dir``: web server, serve files from a dir + special filename for file
  list
- ``C.dbs.git-ssh``: openssh + mounted git repo
- ``C.vs.pdf-page``: web server, input: (pdf filename, page #), output: image
- ``C.ca.mem``: input: web request, output: result from cache or querieng this
  data from ``C.sts.dir``, ``C.vs.pdf-page``
- ``C.se.pdf-page``: input: string, output: list[pdf file, page #]
- ``C.re.search``: input: search string, output: web page with search string +
  results
- ``C.pr.all-records``: web server, input: record UUID, output: record


Later
-----

- nginx as a web server
- git for metadata
- filesystem for records
- rst file format
- rst2html5 renderer
- vim to modify

TODO
====

- a script to generate & check copyright header for all files in the repo
- CI to check everything pre-commit & post-commit
- CI to create all the packages and create GitHub releases


API
===

Storage Server
--------------

- operations: get, put, head, list, delete

  - list

    - instance: 'any', 'all', UUID or nothing
    - record: record UUID to get info about specific record, nothing to get all
      records
    - version: version UUID to get a specific record version info, nothing or
      'all' to get all record versions
    - result: json with all record that match criteria. Empty dict if there
      are no such records.

  - put

    - instance: 'any', 'all', UUID or nothing
    - record: record UUID (to put a specific record) or nothing (to assign
      new UUID)
    - version: nothing (for now)

  - get

    - instance: 'any', 'all', UUID or nothing
    - record: record UUID
    - version: version UUID or nothing (to get the latest version)

- parameters

  - instance

    - UUID: specific instance UUID
    - absent: this instance (or any instance for some cases)
    - any: any instance is fine
    - all: (for list, delete) apply to all instances

  - record

    - UUID: record UUID
    - absent: (for list) any record

  - version

    - UUID: version UUID
    - absent: latest version
    - all: all versions

- future operations

  - stats - get storage server statistics
  - info - get configuration etc.
  - fsck - execute fsck
  - health - health check


Record Server
-------------

- operations on tags and attributes
- tag: a string
- attribute: kv pair
- limitations: tags, keys and values MUST NOT have '\n' inside
- tag API: add, del, get
- attribute API: set, del, get


Existing implementations
========================

Articles
--------

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
------------

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
- https://obsidian.md/features
- https://www.dendron.so/

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

Reading list
============

- https://typesense.org/about/
- https://en.wikipedia.org/wiki/Uniform_Resource_Identifier#Syntax
- https://en.wikipedia.org/wiki/Key%E2%80%93value_database
- https://c4model.com/
- https://github.com/junegunn/fzf
- https://scribe.rip/p/what-every-software-engineer-should-know-about-search-27d1df99f80d


Ideas
=====

- ``C.manager`` also collects all the logs and makes them available as records
- The original Zettelkasten as a test data
- query like "choose:date", and the datepicker appears below. "choose:contry",
  and the list of all attributes country=something appear bellow, which allows
  filtering by clicking on them.
