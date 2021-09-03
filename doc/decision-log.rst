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
  - C.record-server

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
