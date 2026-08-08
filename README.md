<img width="843" height="378" alt="banner" src="https://github.com/user-attachments/assets/bd66a8b3-683f-425c-b16a-f04510d1952d" />

# Refracted

Refracted brings cancelled, inactive, and historical games and builds back online — titles that lost their backend and stopped working when publisher services shut down or never shipped.

Site: [refracted.au](https://refracted.au/)

### How it works

There's a **central Refracted host** for the supported catalogue. Refracted (accounts) players connect there for titles that are already wired up.

Developers and communities can also run a **local Refracted instance** to stand up service layers, test connectivity, and prove a title works. When it's ready, that work can be brought into the official catalogue for hosted support.


### Prism

Client redirect / local routing uses **Prism**, a companion project. Source is private; compiled game-specific builds for supported titles are coming and will be distributed directly via the Refracted launcher (coming soon).

### Games

| Game | Notes |
|------|--------|
| **Command & Conquer** | In progress — Aurora |
| **Battlefield 3 (Alpha / Beta)** | Planned |
| **Battlefield Labs** | Future Planned / Parked |

More titles get added as local work lands and is promoted into the catalogue. See [refracted.au](https://refracted.au/) for what's live.

### Media

<img width="1201" height="831" alt="Refracted" src="https://github.com/user-attachments/assets/10533d00-2a7e-4eee-83d5-f5289906f145" />

<br>

<img width="1199" height="833" alt="Refracted 2" src="https://github.com/user-attachments/assets/940c03bd-4853-4613-85f3-c4a612dffdca" />

### Disclaimer

Independent community project. Not affiliated with EA or any rights holder. Game names are their trademarks.

For education, research, and preservation. Not official software. Use it legally and within the terms of whatever game software you own. No piracy, cheating, or messing with live services.

### Build (Windows)

Needs a recent stable Rust toolchain ([rustup](https://rustup.rs/)).

```bash
cd refracted
cargo build --release
cargo run --release
```
