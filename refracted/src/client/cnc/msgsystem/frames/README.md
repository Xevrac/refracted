# MessageSystem wire frames (optional local goldens)

This directory may hold **pre-serialized TCP frames** used only as dump-parity
test goldens. They are **not** required to clone, build, or run Refracted.

Handshake bytes that the emulator actually sends (`ProtocolVersion`, `ServerHello`,
`LoadMap`, `ServerReadyToStart`) are encoded in Rust (`negotiation.rs` /
`messages.rs`). Runtime does **not** load game DLLs and does **not** embed `*.bin`.

## Production ownership

| Role | Owner |
|------|--------|
| Client MsgSys TCP dial | Client → Refracted `:18386` (MITM) |
| ServerHost join + gameplay | **Prism** `prism.cnc.network.dll` on dedicated `:18387` |
| SimuCloud CreateGame | Refracted orchestrator → dedicated `:18388` |

Do **not** run an embedded Refracted ServerHost for production joins.


```bash
cargo build --release
```

`cargo test` dump-parity cases (`*_matches_retail_dump`, SimuCloud PTM parse)
**skip** when the matching `*.bin` is absent. Encoder shape tests always run.

## Not in git (regeneration artifacts)

| Path | Contents |
|------|----------|
| `*.bin` | Optional wire goldens (output of a private dumper) |
| `dlls/` | Retail game DLLs copied from a local install |

Do not commit, redistribute, or share `dlls/` (copyrighted EA material). The
dumper that produces `*.bin` is a private local tool — it is **not** in this
repository and is not needed to build.

## Optional: local dump-parity goldens

Maintainers who already have a licensed C&C 2013 install and the private dumper
can drop `*.bin` here so dump-parity tests compare Rust encoders against retail
serialization. Everyone else can ignore this folder.

Typical golden names (when present):

| File | Role |
|------|------|
| `client_protocol_version.bin` | Client-channel `ProtocolVersion` (v2 + empty auth) |
| `load_map.bin` | `LoadMap` (default tutorial map) |
| `server_hello.bin` | `ServerHello` |
| `server_ready.bin` | `ServerReadyToStart` |
| `simucloud_ptm.bin` | SimuCloud channel PTM |
| `ptm.bin` | Client-channel `ProtocolTypeIdMapping` (unused at runtime) |

`dlls/` is only for regenerating goldens, never for `cargo build`.
