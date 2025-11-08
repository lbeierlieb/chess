# Chess Bevy Game 3D

Exactly what the title says: a chess implementation using the Bevy game engine for 3d visualization.

Try for yourself in the browser: https://lbeierlieb.github.io/chess/

![](cover.png)

# Features and Limitations

Existing functionality:
- Interacting with the chess board with mouse clicks
- Display of possible moves
- En Passant and Castling
- Winner detection

Current limiations:
- Winner is only printed in log
- No detection of draw
- Castling possible in cases where it shouldn't
- Always showing perspective of player White

# Run locally

You can run the native build by executing:
```
nix run github:lbeierlieb/chess
```

You can a python webserver serving the WASM build with:
```
nix run github:lbeierlieb/chess#chess-wasm
```

# Develop

The nix builds are not recommended for builds during development, as the build times are annoyingly long.
Use the nix development shell to build directly with cargo instead.
In the checked out repository, run:
```
nix develop
cargo run
```
Dynamic linking is active and allows rebuilding within a few seconds.
