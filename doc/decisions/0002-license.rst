..
    SPDX-License-Identifier: AGPL-3.0-or-later

    mze - personal knowledge database
    Copyright (C) 2026  Maksym Medvied

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


.. list-table::

   * - status
     - accepted
   * - introduced
     - 2026-03-29
   * - last updated
     - 2026-03-29
   * - decision-makers
     - Maksym Medvied
   * - consulted
     -
   * - informed
     -


License
=======

Context and Problem Statement
-----------------------------

Need to choose a license.


Decision Drivers
----------------

- mze is going to use a lot of plugins for all kinds of software. It makes
  sense to have a license that is compatible with the most of the FOSS licenses
  out there.
- I'm making a repo on codeberg, it's a good time to change the license if the
  change is needed.


Considered options
------------------

- AGPL-3.0-or-later
- GPL-3.0-or-later
- GPL-2.0-only
- Apache-2.0


Decision Outcome
----------------

- AGPL-3.0-or-later


Consequences
------------

- Good, because it's FOSS.
- Good, because it's the best for the mze use case according to the FSF.
- Bad, because it's not compatible with GPLv2-only and I'll need to license the
  code that needs to work closely with GPLv2-only code under 2 licenses.
- Bad, because in some cases people may not be able to use mze due to the
  company-wide restrictions on GPL licenses.


Confirmation
------------

LICENSE will contain the text of AGPLv3.


Pros and Cons of the Options
----------------------------

GPL-3.0-or-later
................

- Good, because it's as good as AGPLv3 if you don't consuder the network use
  case
- Bad, because in some cases people may not be able to use mze due to the
  company-wide restrictions on GPL licenses.


GPL-2.0-only
............

- Good, because it's compatible with GPL-2.0-only
- Bad, because AGPL-3.0-or-later or GPL-3.0-or-later is better in 2026 (IMO).


Apache-2.0
..........

- Good, because there is nothing to relicense.
- Bad, because a lot of code to interact with other software will need to be
  dual-licensed.


More information
----------------

- I'm not a lawyer, nowhere here is a legal advice, etc.
- https://www.gnu.org/licenses/license-recommendations.html
  How to Choose a License for Your Own Work
- https://runxiyu.org/comp/gplproxy/
  https://news.ycombinator.com/item?id=47272534
  GPL upgrades via section 14 proxy delegation
- https://spdx.org/licenses/
  SPDX License List
- https://www.gnu.org/licenses/gpl-howto.html
  How to Use GNU Licenses for Your Own Software
