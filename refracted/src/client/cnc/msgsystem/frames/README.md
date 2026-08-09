# MessageSystem wire frames

This directory holds **pre-serialized TCP frames** for the C&C 2013 `Rts.Messaging` handshake and
bootstrap control messages. Refracted embeds them at compile time (`include_bytes!` in
`negotiation.rs`) for SimuCloud negotiation and wire-shape tests.

## Production ownership

| Role | Owner |
|------|--------|
| Client MsgSys TCP dial | Client → Refracted `:18386` (local redirect) |
| ServerHost join + gameplay | **Prism** `prism.cnc.network.dll` on dedicated `:18387` |
| SimuCloud CreateGame | Refracted orchestrator → dedicated `:18388` |

Do **not** run an embedded Refracted ServerHost for production joins.

## Not in git

| Path | Contents |
|------|----------|
| `*.bin` | Generated wire frames (local build output) |
| `dlls/` | Local staging for optional frame tooling (gitignored; use your own lawful install) |

Do not commit, redistribute, or share these.

## Regenerating frames

When wire defaults change, regenerate embedded frames with the local frame tooling under
`tools/` (if present in your tree), then rebuild Refracted:

```bash
cargo build --release
```

Frames are compiled into the emulator binary. Runtime does **not** load game libraries.

## When to regenerate

- After a retail client patch changes join handshake wire shapes
- When changing defaults used by the local frame tooling
