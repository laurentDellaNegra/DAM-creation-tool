# DAM Creation Tool

Rust + egui implementation of the DAM Creation Tool.

The app targets both native desktop and web/WASM from one shared egui UI. The
first version focuses on static predefined map creation with deterministic JSON
export. AIXM export will be added later once the template is available.

## Layout

```text
.
├── apps/
│   ├── native/   # desktop launcher
│   └── web/      # Trunk/WASM launcher
├── assets/
│   ├── favicon.svg
│   ├── maps/     # bundled GeoJSON static maps
│   ├── pmtiles/  # runtime/static PMTiles assets
│   └── templates/
├── crates/
│   ├── dam-core/ # domain model, validation, catalog parsing, export
│   └── dam-egui/ # shared egui app
├── docs/
├── prompts/
└── scripts/
    └── dam       # local command runner
```

## Commands

Most project commands can be launched through the local command runner:

```sh
./scripts/dam help
```

Common commands:

```sh
./scripts/dam setup
./scripts/dam check
./scripts/dam test
./scripts/dam fmt
./scripts/dam fmt-check
./scripts/dam clippy
./scripts/dam native
./scripts/dam native-build
./scripts/dam native-release
./scripts/dam web
./scripts/dam web-build
./scripts/dam maps-refresh
./scripts/dam maps-count
./scripts/dam verify
```

Install the Rust WASM target for the web app:

```sh
rustup target add wasm32-unknown-unknown
```

Install Trunk for web development:

```sh
cargo install trunk
```

Check the full workspace:

```sh
cargo check --workspace
```

Run tests:

```sh
cargo test
```

Format Rust code:

```sh
cargo fmt --all
```

Run Clippy:

```sh
cargo clippy --workspace --all-targets
```

Run the native app:

```sh
cargo run -p dam-native
```

Build the native app:

```sh
cargo build -p dam-native
```

Build the native app in release mode:

```sh
cargo build -p dam-native --release
```

Run the web app locally:

```sh
cd apps/web
trunk serve
```

Build the web app:

```sh
cd apps/web
trunk build
```

If Trunk receives `NO_COLOR=1` from the shell and rejects it, unset that variable:

```sh
cd apps/web
env -u NO_COLOR trunk serve
env -u NO_COLOR trunk build
```

Refresh the selected bundled maps from the local ACWP static data checkout:

```sh
./scripts/dam maps-refresh
```

Override the source or destination if needed:

```sh
MAP_SOURCE=/path/to/maps MAP_DEST=assets/maps ./scripts/dam maps-refresh
```

Verify the bundled map count:

```sh
./scripts/dam maps-count
```

## Map Assets

Place bundled static maps in `assets/maps/` as GeoJSON `FeatureCollection`
files.

Catalog rules:

- Map id comes from the filename stem, for example `50714.geojson` -> `50714`.
- Map name comes from the top-level `name`.
- Top-level `description` is used for search/detail text.
- Static map export includes only the map id and name, not GeoJSON geometry.
