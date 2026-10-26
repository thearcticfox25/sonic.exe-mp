## About Sonic.EXE Multiplayer

Sonic.EXE Multiplayer is Team EXE Empire's [Sonic.EXE: TD2DR](https://gamejolt.com/games/disaster2dr/780541) ported to Rust, with its server in the same program. It started as a server for TD2DR; now it is the game, and running it is all a player has to do. The server and the game share one set of resources and one simulation (the physics, the levels, the characters, the gameplay numbers). That shared base is the groundwork for an anticheat that is part of the foundation rather than a check bolted on from outside.

The server runs every round itself, on the same code the game uses to predict its own player. A client sends only its buttons, and the server decides what they do. Nothing a client says about positions, hits, rings or objects is taken as true, so there is nothing to lie about.

**License:** AGPL v3 (see `LICENSE.md` for the licenses of the parts that come from other projects)

## One program, two roles

```
AdvancedServer [--play]            the game (a double click starts it; --client and --game too)
AdvancedServer --host              the server (--server and --dedicated too)
AdvancedServer --update            download the latest release, asking first
AdvancedServer --help              every option
```

- **Game:** TD2DR, ported from its GameMaker project. It plays online on a Sonic.EXE Multiplayer server or alone on any map.
- **Server:** lobbies, voting, the rounds themselves, moderation, and an integrated terminal for administration. It is configured in `PersistentStorage/Server/Config.toml`.
- **Shared:** `Resources/` (textures, sounds, maps, fonts) and `PersistentStorage/Core/` (the gameplay numbers: physics, characters, levels). The server sends its numbers to every player at the start of a round, so everyone plays by the server's rules.

## Compatibility

The TD2DR client cannot join: the protocol changed from "the client reports, the server checks" to "the client sends input, the server simulates". Play with this program's own game: just run it.

## Commands

Commands are prefixed based on their purpose and origin:
- `.` - TD2DR player commands
- `:` - new player commands added in Sonic.EXE Multiplayer
- `?` - administrative commands (system administrator only)

Use `.help` in the lobby, or `?help` in the terminal, for a list of available commands.

## Building

On Linux the sound library (cpal) needs the PipeWire and ALSA headers, and clang to read
them; the gamepad library (gilrs) needs the libudev headers (systemd-devel). On Fedora:

```
sudo dnf install gcc pipewire-devel alsa-lib-devel clang systemd-devel
```

Rust itself comes from [rustup](https://rustup.rs), since `Cargo.toml` asks for a newer
version than a distribution may ship. Then:

```
cargo build --release
```

A release build packs `Resources/` into `Resources.tar` next to the binary; a debug build copies the folder there. `PersistentStorage/` is written on the first start.

## Credits

Most of this project is Team EXE Empire's work. [Sonic.EXE: TD2DR](https://gamejolt.com/games/disaster2dr/780541) is theirs, and its GameMaker source is published under the MIT license (`Licenses/MIT-TeamExeEmpire.txt`). From it come:
- the game itself: every object, script, room and shader of TD2DR, ported to Rust (the client, and the simulation the server runs);
- everything in `Resources/`: the sprites, sounds, music and maps, converted from the TD2DR project;
- the network protocol, from their [BetterServer-OSS](https://github.com/teamexeempire/betterserver-oss) (MIT license).

On top of it, Sonic.EXE Multiplayer brings the architecture that conducts these scripts and resources (one program for the server and the game, the shared simulation, the server-run rounds and their protocol), and the new options and game modes not credited below.

The people behind TD2DR (its code, level design, sprites, music, voice acting and testing) are listed in its credits, which this game shows as they were. Part of the sprites and music are by third parties the TD2DR credits name (Sonic Team among them).

Also:
- The Overhaul game type comes from the [Gamemode Cycle](https://gamejolt.com/games/TD2DRgmcycle/919987) server mod, created by IcedCoffee & ColdestTea.
- The disable_timer function is a reimplementation of Banana8870's [TD2DR No Timer](https://gamejolt.com/games/no_timer_td2dr/923609) server mod.
- Networking is built on [ENet](https://github.com/lsalzman/enet) (MIT license) by Lee Salzman, via its Rust port [rusty_enet](https://github.com/jabuwu/rusty_enet) (MIT license) by jabuwu.
- The per-tick event/packet flood cap is adapted from vinny's disasterserver, used here with permission.
- The variable jump (as high as jump is held) comes from TD2D Sonic MOD.
- Spring inertia and rings that scatter on a hit are not_catfish_xd's ideas, the springs from his own mod, Advanced Client.

---

**!!! NOTICE:** This version is unstable. Use at your own risk in production environments.
