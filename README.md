<img width="843" height="378" alt="banner" src="https://github.com/user-attachments/assets/bd66a8b3-683f-425c-b16a-f04510d1952d" />

# Refracted

**Refracted** is a desktop program that runs on your own computer. It acts as a *local stand-in* for the online services that certain games talk to when they connect to publisher infrastructure. Instead of those remote systems, Refracted answers on your machine so you can study how the client behaves, document behavior, or keep access working for titles that are no longer fully supported online.

### Our mission

Refracted's purpose is to preserve video games against time. Examples have arisen over time where we see core infrastructure is shutdown and games no longer function at all as a result for one reason or another.

### Related tooling

Refracted emulates **backend** services on your machine. The game client must still be directed to that machine (for example via hosts or DNS, or client-side patching). **Prism** is a separate companion effort focused on client-side hooks and patching for workflows that use Refracted; its **source is not in this repository**. Any prebuilt Prism binaries are distributed on their own release channels when available.

### Client usage

You can run Refracted on your own PC and redirect traffic to `localhost` and have a locally running game. Ideal for researchers, small communities or peer to peer multiplayer where centralising data does not really matter.

### Server usage

You can run Refracted from a server and have your project redirect traffic to the `IPv4` address and have a centrally wired game; this would be common for a game needing central data distributed like leaderboards or stats between multiplayer matches / games.

> Disclaimer: The project is **work in progress**. Behavior and features will change as development continues.

## What you can use it for now

- **Run services in client or server architecture** — Start the emulator from the app and let a compatible game client connect to Refracted instead of live backend hosts.
- **Choose a game profile** — Pick which title you are emulating so ports and behavior match that game’s expectations.
- **Learn and preserve** — Useful for understanding client–server flows and for preservation when official services are unavailable or changed.

### Development

Refracted comes with its own toolset for research and development.
* **Listen** — Captures Blaze, gRPC, HTTP, and LSX traffic for analysis (see Technical details).
* **Make** — Work-in-progress Blaze and gRPC payload workbenches, with Blaze **live injection** for controlled tests.
* **Dump** — Save captures to files for offline review (paired with **Listen**).

Refracted **does not replace a full commercial backend and is not intended for this**; it implements enough *service layers* (see the technical section) to support the scenarios the project targets in the interest of our mission.

## Games supported today

These profiles ship as defaults (you can adjust the list in your local configuration if the app creates one):

| Game | Notes |
|------|--------|
| **Battlefield Labs** | Full set of emulated service layers enabled in the default profile. |
| **Command & Conquer** | Select set of service layers enabled in the default profile; tuned for that title’s connectivity pattern. |

If a game is not listed here, it is not part of the default supported set yet.

# Media 

<img width="1201" height="831" alt="Refracted" src="https://github.com/user-attachments/assets/10533d00-2a7e-4eee-83d5-f5289906f145" />

<br>

<img width="1199" height="833" alt="Refracted 2" src="https://github.com/user-attachments/assets/940c03bd-4853-4613-85f3-c4a612dffdca" />

## Important disclaimer (EA, titles, and software)

**Refracted is an independent, community project.** It is **not** affiliated with, endorsed by, or sponsored by Electronic Arts Inc. or any other rights holder. **Battlefield**, **Command & Conquer**, and other game names are trademarks of their respective owners.

This software is provided for **education, research, and preservation**. It is **not** official software and **not** a product of EA or any publisher. The authors make no claim on publisher code, assets, or proprietary protocols beyond what is necessary to interoperate at a high level for the stated purposes.

You are responsible for using Refracted **only in ways that comply with applicable law** and with the terms that apply to software you use (including game end-user agreements). The project maintainers do not encourage or support piracy, cheating, or circumventing technical protections for live service titles or live infrastructure layers.

---

## Technical details

### Layers and protocols

Refracted is implemented in **Rust** and is organized around **service layers** that mimic parts of EA-style online stacks:

- **Blaze** — Binary Blaze/frostbite-style game services (including redirector and related listeners as configured per title).
   - Refracted has support for FireFrame and Fire2Frame built-in.
- **Web (HTTP/HTTPS)** — Web API surfaces; **gRPC-over-HTTP/2** used by some titles is handled here in addition to standard HTTP/HTTPS content backend as a webserver.
- **LSX** — LSX-style listener where enabled for a profile.
- **QoS** — Quality-of-service style endpoints.
- **RTM** — RTM listener where enabled.

Per-title settings also record **wire protocol variants** (e.g. **Fire2Frame** vs **FireFrame**) and build identifiers used to select behavior. **TLS** may be used for redirector traffic depending on the profile.

### Source code layout

The repository is organized so **shared protocols** stay separate from **title-specific** behavior:

- **`refracted/`** — The Rust crate: library (`lib.rs`) plus the desktop binary (`main.rs`).
- **Service layers** — Folders such as `blaze/`, `http/`, `web/`, `lsx/`, `qos/`, and `rtm/` each own listeners and handlers for that part of the stack. Shared helpers live in `grpc/`, `jwt/`, `session/`, and `crypto/`.
- **`client/`** — Per-game modules (for example `labs/` for Battlefield Labs, `cnc/` for Command & Conquer). Dispatch goes through `client/mod.rs` based on the **current game id** from configuration.
- **`common/`** — Cross-cutting pieces: game registry and ports (`game/`), paths, settings, errors, and user profiles.
- **Configuration** — Default title list is seeded from `refracted/resources/default_games.json` into a user-editable `games.json` under the app data path.

The UI in `main.rs` is intentionally thin: it starts the async server and hosts **egui** windows; protocol logic remains in the library so it can be reused or tested without the GUI.

### Toolkit (research and development)

The in-app **toolkit** is aimed at **documentation and research**: observing what clients send, comparing to live services, and experimenting with messages.

- **Two inspection modes**
  - **Emulator** — Records traffic that flows through Refracted while you use the local emulator.
  - **Research (proxy)** — Forwards the client toward **upstream** services while copying traffic into the same capture buffers, so you can compare emulator behavior with live responses. Optional **proxy listeners** (HTTP/HTTPS, gRPC, Blaze, LSX) are configurable in the UI when this mode is active.
- **Listen** — Tabbed viewers for **Blaze**, **gRPC**, **HTTP**, and **LSX** captures (timestamps, directions, payloads). You can export or save captures for offline review.
- **Make** — Workbenches to **compose or inspect** Blaze and gRPC payloads (including presets and structured views where implemented). Blaze supports **live injection** to connected clients for controlled experiments.
- **Labs capture replay** — For supported gateway URL patterns, optional **replay** of stored HTTP bodies can load from local `data/` files (see the `client/labs/capture` module for rules).

The HTTPS proxy path is **partially implemented**; full TLS interception would require local certificate setup beyond what Refracted ships today.

### Building from source

> Note this software is currently built and intended for use in Windows at this point in time.

1. Install a recent **stable Rust** toolchain ([rustup](https://rustup.rs/)).
2. From the repository root, use the crate under `refracted/`:

```bash
cd refracted
cargo build --release
```

Run the desktop app:

```bash
cargo run --release
```

Debug builds work with `cargo build` / `cargo run` without `--release` but are slower and note these take up a considerable amount of space.

On Windows, the project is typically built and run from a developer shell (PowerShell or cmd) with `cargo` on your `PATH`.
