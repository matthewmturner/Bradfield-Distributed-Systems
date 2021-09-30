# Blue - Distributed Key-Value Store

## Summary

Blue was created as the core project for the Bradfield Distributed Systems course. The project was to create a distributed Key-Value store from scratch while considering the various design decisions that come with build such a system.

## Core Features

- Implemented in Rust
- Client / Server architecture
  - Client requires the provided Blue client to be run (i.e. you can't use `netcat` or something similar) so that Protocol Buffers messages can be sent.
  - Multiple servers can be run and act as a cluster
- Accepted commands
  - `set`: Set a single key value pair. e.g. `set name=matt`
  - `get`: Get the value for a single key e.g. `get matt`
  - `delete`: Delete the value for a single key e.g. `delete matt`
  - `get_from`:
  - `change_store`:
- Transport Layer Protocol is TCP
- Serialization format for both client / server and on disk storage is Protocol Buffers
- On disk storage
  - The entire store is rewritten to disk after each `set`
  - A Write-Ahead-Log is updated after each `set` command to enable more efficient backup / synchronization
- Write-Ahead-Log
  - Format:
    - Header
      1. 4 magic bytes "BLUE"
      2. 1 byte for which version of the WAL this is
      3. 1 byte for which version of Protocol Buffers is used
    - Data
      1. 8 byte sequence number
      2. 4 byte Protocol Buffers message length
      3. Protocol buffers message
- Replication is semi-synchronous
  - Leader and first follower are synchronous
  - All subsequent followers are asynchronous

## User Guide

### Rust Installation

###
