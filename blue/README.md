# Blue - Distributed Key-Value Store

## Summary

Blue was created as the core project for the Bradfield Distributed Systems course. The project was to create a distributed Key-Value store from scratch while considering the various design decisions that come with build such a system.

## Core Features

- Implemented in Rust
- Client / Server architecture which requires the provided Blue client to be run (i.e. you can't use `netcat` or something similar)
- Accepted commands
  - `set`: Set a single key value pair. e.g. `set name=matt`
  - `get`: Get the value for a single key e.g. `get matt`
  - `delete`: Delete the value for a single key e.g. `delete matt`
- Serialization format for both client / server and on disk storage is Protocol Buffers
- On disk storage
  - The entire store is rewritten to disk after each `set`
  - A Write-Ahead-Log is generated after each `set` command to enable more efficient backup / synchronization
- Write-Ahead-Log
  - Format:
