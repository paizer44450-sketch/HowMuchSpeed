# How Much Speed – source code

Source code of **How Much Speed**, an Elden Ring boss rush mod: talk to Melina at the
Roundtable Hold, pick a boss of Elden Ring or Shadow of the Erdtree, set the game speed
from 100% to 300%, customise your visual effects, and fight.

**Players: download the ready-to-use mod on Nexus Mods**, not this repository.
This repository only contains the code of the DLL (`vitesse_boss.dll`). The game files
the mod also needs (regulation.bin, map, texts, effects) belong to FromSoftware and are
only distributed in the Nexus archive.

Offline only. The mod is loaded by Mod Engine 2, without Easy Anti-Cheat.

## Building

Requirements: Windows, [Rust](https://rustup.rs) (stable, MSVC toolchain).

```
cargo build --release
```

The DLL is written to `target/release/vitesse_boss.dll`. Put it next to
`modengine2_launcher.exe`, with `external_dlls = [ "vitesse_boss.dll" ]` in
`config_eldenring.toml`, and the mod files from the Nexus archive in the `mod` folder.

Code comments and in-game logs are in French.

## Libraries

- [fromsoftware-rs](https://github.com/vswarte/fromsoftware-rs) – MIT NON-AI / Apache 2.0 NON-AI
- [hudhook](https://github.com/veeenu/hudhook) and Dear ImGui – MIT
- [windows-rs](https://github.com/microsoft/windows-rs) – MIT / Apache 2.0

Full license texts: `THIRD-PARTY-LICENSES.txt`.

## AI disclosure

Most of this code was written with an AI assistant (Claude, by Anthropic), directed
and tested by the author.

## License

MIT for the code in this repository (see `LICENSE`).
