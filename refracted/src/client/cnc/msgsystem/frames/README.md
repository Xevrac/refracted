# MessageSystem wire frames (local dev artifacts)

This directory holds **pre-serialized TCP frames** for the C&C 2013 `Rts.Messaging` handshake and
bootstrap control messages. Refracted embeds them at compile time (`include_bytes!` in
`negotiation.rs`) for SimuCloud negotiation and dump-parity tests.

## Production ownership

| Role | Owner |
|------|--------|
| Client MsgSys TCP dial | Client → Refracted `:18386` (MITM) |
| ServerHost join + gameplay | **Prism** `prism.cnc.network.dll` on dedicated `:18387` |
| SimuCloud CreateGame | Refracted orchestrator → dedicated `:18388` |

Do **not** run an embedded Refracted ServerHost for production joins.

## Not in git

| Path | Contents |
|------|----------|
| `*.bin` | Generated wire frames (output of dumper) |
| `dlls/` | Retail game DLLs copied from your local install |

Do not commit, redistribute, or share these (copyrighted EA material).

## First-time setup (per developer)

You need a **local** C&C 2013 install.

### 1. Stage retail DLLs

Copy from your game `Bin\Command & Conquer\` into **`dlls/`** (this folder, gitignored):

```
frames/dlls/
  Serialization.dll
  PlayerMessages.dll
  SlimMath.dll
```

### 2. Generate frames

```bash
cd tools/msgsys-dump
dotnet run -c Release
```

Writes `*.bin` into this directory:

| File | Role |
|------|------|
| `client_protocol_version.bin` | Client-channel `ProtocolVersion` (v2 + empty auth) |
| `ptm.bin` | `ProtocolTypeIdMapping` reply (ServerHost negotiation) |
| `load_map.bin` | `LoadMap` (default tutorial map) |
| `server_hello.bin` | `ServerHello` |
| `server_ready.bin` | `ServerReadyToStart` |
| `simucloud_ptm.bin` | SimuCloud channel PTM |

### 3. Build Refracted

```bash
cargo build --release
```

Frames are compiled into the emulator binary. Runtime does **not** load game DLLs.

## When to regenerate

- After a retail patch changes `PlayerMessages` or `Serialization`
- When changing defaults in `tools/msgsys-dump/Program.cs`

See `tools/msgsys-dump/README.md` for tool details.
