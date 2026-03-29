.. list-table::

   * - status
     - accepted
   * - introduced
     - 2026-03-28
   * - last updated
     - 2026-03-28
   * - decision-makers
     - Maksym Medvied
   * - consulted
     -
   * - informed
     -

Using architectural decisions records
=====================================

Context and Problem Statement
-----------------------------

I need to record project decisions.


Decision Drivers
----------------

- doc/decision-log.rst is not enough for complex projects. I think it wouldn't
  scale well past 100 records. mze would solve this anyway, but until mze is
  ready it makes sense to have something that is ready to be moved to mze.


Considered options
------------------

- mze itself
- doc/decision-log.rst
- MADR: https://github.com/adr/madr
- ISO/IEC/IEEE 42010 template: http://www.iso-architecture.org/42010/templates/
- Structured MADR: https://smadr.dev/


Decision Outcome
----------------

- MADR-based .rst


Consequences
------------

- Good, because it's easy to change to something else when needed
- Bad, because it's not something standard => people are not familiar with it
- Bad, because there are not tools to work with it


Confirmation
------------

This file confirms that the process is started.


Pros and Cons of the Options
----------------------------

mze itself
..........

- Good, because it's mze
- Bad, because it's not implemented right now


doc/decision-log.rst
....................

- Good, because it's simple
- Bad, because it doesn't scale


MADR
....

- Good, because it's something that is well-documented
- Neutral, because it's Markdown and I don't want to mix .rst and .md here


ISO/IEC/IEEE 42010 template
...........................

- Good, because it's an international standard
- Bad, because the standard is behind a paywall
- Bad, because it's not clear how to use CC-BY 3.0 (the license of the
  template) here


Structured MADR
...............

- Good, because it's machine-readable
- Bad, because right now I don't need it


More information
----------------
