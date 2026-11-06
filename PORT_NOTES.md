# Port notes: bevy 0.16

This branch ports the demos to the current `bevy` branch of AtomECS, which is
built against **bevy 0.16.1**. The original code targeted bevy 0.9 and no longer
compiles against the engine, because `atomecs` floats on `branch = "bevy"` and
that branch was upgraded in September 2025.

## What changed

| Old (bevy 0.9) | New (bevy 0.16) |
| --- | --- |
| `bevy = "0.9"` | `bevy = "0.16"` |
| `bevy_egui = "0.19"` | `bevy_egui = "0.34"` |
| `nalgebra = "0.31"` | `nalgebra = "0.33"` |
| `rand = "0.8"`, `rand_distr = "0.4"` | `rand = "0.9"`, `rand_distr = "0.5"` |
| `app.add_plugin(P)` | `app.add_plugins(P)` |
| `app.add_system(f)` | `app.add_systems(Update, f)` |
| `app.add_startup_system(f)` | `app.add_systems(Startup, f)` |
| `WindowDescriptor` | `Window` inside `WindowPlugin { primary_window }` |
| `PbrBundle { .. }` | `(Mesh3d(..), MeshMaterial3d(..), Transform)` |
| `Camera3dBundle` + `OrthographicProjection` field edit | `Camera3d` + `Projection::Orthographic(ortho_camera(..))` (helpers in `src/camera.rs`, `OrthographicProjection` has no `Default` in 0.16) |
| `DirectionalLightBundle` / `PointLightBundle` | spawn `DirectionalLight` / `PointLight` components directly |
| `shape::Icosphere { .. }` / `shape::Capsule { .. }` | `Sphere::new(r)` / `Capsule3d::new(r, h)` |
| `Input<MouseButton>` | `ButtonInput<MouseButton>` |
| `Res<Windows>` window query | `Query<&Window, With<PrimaryWindow>>` |
| `EventReader<E>::iter()` | `.read()` |
| `SceneBundle { scene: handle, .. }` | `(SceneRoot(handle), Transform)` |
| `visibility.is_visible = b` | `*vis = if b { Visibility::Inherited } else { Visibility::Hidden }` |
| `TextBundle` / `TextStyle` / `Style` UI text | `Text`, `TextFont`, `TextColor`, `Node` components |
| `Color * f32` | `scale_color(c, f)` helper (`src/atoms.rs`); `emissive = color.to_linear() * strength` |
| `Color::BLUE` etc. | `Color::srgb(..)` |
| `color.set_a(a)` | `color.with_alpha(a)` |
| `EguiContext` param, `.iter()` panels | `EguiContexts`, `let ctx = ctxs.ctx_mut().clone();` then `.show(&ctx, ..)` |
| `EguiPlugin` | `EguiPlugin { enable_multipass_for_primary_context: false }` |
| `window.set_mode(..)` | `window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current)` |
| `rand::thread_rng()` | `rand::rng()` |
| `atomecs ... CalculateActualPhotonsScattered` system label | `.after(laser_cooling::LaserCoolingSystemsSet::Set)` |
| `app.world` | `app.world_mut()` |

The `schreck` example's embedded magnetic field JSON is now a Rust raw string
(`r#"..."#`) instead of escaped string content.

## Wasm (GitHub Pages) note

`getrandom` 0.3 renamed its wasm feature from `js` to `wasm_js`, and crates.io
patches must point at a different source, so the old `[patch.crates-io]` entry
in the manifest was removed. Instead, wasm is enabled with two things:

1. In `Cargo.toml`:

```toml
[target.wasm32-unknown-unknown.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```

2. In `.cargo/config.toml` (new file, checked into the repo): getrandom 0.3
   additionally requires the `getrandom_backend="wasm_js"` cfg flag, which the
   feature alone does not set. Without it, wasm builds fail with
   `The wasm32-unknown-unknown targets are not supported by default`.

```toml
[target.wasm32-unknown-unknown]
rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
```

## Local wasm testing: the `#bevy` canvas

The demos create their window with `canvas: Some("#bevy")`, matching the HTML
pages in `pages/` that are deployed to GitHub Pages (each contains
`<canvas id="bevy"></canvas>`). `wasm-server-runner`'s default page has no such
element, so a local browser run panics with `Cannot find element: #bevy`
followed by `Unable to find a GPU!` (the second error is a cascade).

Fix shipped with this port:

- `web/index.html` is a copy of wasm-server-runner's default page with the
  `<canvas id="bevy"></canvas>` element added (its `{{ NO_MODULE }}` /
  `// {{ MODULE }}` placeholders are kept so the runner still injects its
  loader and console forwarding).
- `.cargo/config.toml` sets the env var the runner reads:

```toml
[env]
WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML = "web/index.html"
```

Run from the repo root so the relative path resolves:

```
cargo run --target wasm32-unknown-unknown --release --example 3d_mot
```

If the env var is not picked up (e.g. older cargo `[env]` handling), set it
manually first: `$env:WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML="web/index.html"`
(PowerShell) or `export WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML=web/index.html`.

## Required one-time step

The old `Cargo.lock` pins bevy 0.9 and must be regenerated after checking out
this branch:

```
git rm Cargo.lock
cargo run --release --example 3d_mot   # regenerates Cargo.lock
git add Cargo.lock
git commit -m "Regenerate Cargo.lock for bevy 0.16"
```
