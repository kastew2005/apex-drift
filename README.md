# APEX DRIFT

A stylized 3D drift-racing game built primarily with Rust and Bevy, targeting browser delivery through WebAssembly and GitHub Pages.

## Prototype controls
- W / Arrow Up: throttle
- S / Arrow Down: brake / reverse
- A / Arrow Left: steer left
- D / Arrow Right: steer right
- Space: handbrake

## Local development
Install stable Rust, add the WASM target and install Trunk:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```

## Production
```bash
trunk build --release
```

## GitHub Pages
Set repository Settings → Pages → Source to GitHub Actions. The workflow builds and deploys `dist`.

The public path is configured as `/apex-drift/`.

## Status
This is the initial foundation. The car is a primitive placeholder and the driving model is an early arcade prototype. Mobile touch controls, finished drift physics, effects, garage, AI and multiplayer are planned next. CI must confirm compilation before this prototype is considered build-verified.
