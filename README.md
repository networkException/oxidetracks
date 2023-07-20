# oxidetracks

An opinionated reimplementation of [owntracks/recorder](https://github.com/owntracks/recorder).

# Opinionated?

This project aims to provide a backend implementation compatible with owntracks clients,
compatibility with the storage format of recorder and a diverging set of features should
be expected in the future.

oxidetracks does not support MQTT

# Running

```
cargo run --release -- --storage-path /path/to/your/owntracks/recorder/storage-directory
```
