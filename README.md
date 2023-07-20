# oxidetracks

An opinionated reimplementation of [owntracks/recorder](https://github.com/owntracks/recorder).

# Opinionated?

While this project aims to provide a backend implementation compatible with owntracks clients,
compatibility with the recorder storage format is expected to break and the set of supported
features will likely diverge.

oxidetracks does not support MQTT.

# Running

```shell
cargo run --release -- --storage-path /path/to/your/owntracks/recorder/storage-directory
```
