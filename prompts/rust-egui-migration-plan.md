# DAM Creation Tool Rust/egui Migration Plan

This document captures the agreed migration decisions from the grilling session.
It is intended to guide the full replacement of the current React/Vite app with a
Rust + egui implementation.

## Goal

Migrate the DAM Creation Tool to Rust and egui, targeting both native desktop and
web/WASM from one shared UI codebase.

The first Rust version implements the current app's scope only, using the PDF in
`docs/` as the behavioral source of truth. It does not attempt to implement the
full DAM management suite from the PDF.

## Scope

In scope for the first version:

- Static predefined map selection from bundled GeoJSON files.
- Map preview in the main creation screen.
- DAM creation form with date range, weekday selection, periods, altitude levels,
  altitude correction, buffer filters, Unit/Sector distribution, text comments,
  and export.
- Strict validation for known DAM/export constraints.
- Deterministic pretty JSON export from a stable Rust domain model.
- Foundation for later AIXM export, once the template is provided.

Out of scope for the first version:

- Real SKYVISU submission.
- AIXM rendering.
- Dynamic/free-drawn maps.
- Map groups.
- Active Maps, Today Maps, Repetitive Maps management.
- Modify/Delete lifecycle.
- Full CWP rendering, transparency/overlap logic, or operational display rules.
- Draft save/load.
- Settings screen.
- Native installers, signing, DMG/MSI packaging.

## Migration Strategy

Replace the current app completely.

Delete the existing Node/Bun/Vite/React/Panda app files, keeping:

- `docs/`
- the existing favicon asset

Move the favicon from `public/logo.svg` to:

```text
assets/favicon.svg
```

Use `assets/` for bundled maps, PMTiles, templates, and retained static assets.

## Project Layout

Use this Rust workspace layout:

```text
.
├── Cargo.toml
├── crates/
│   ├── dam-core/
│   └── dam-egui/
├── apps/
│   ├── native/
│   └── web/
├── assets/
│   ├── favicon.svg
│   ├── maps/
│   ├── pmtiles/
│   └── templates/
└── docs/
```

Responsibilities:

- `dam-core`: domain model, validation, map catalog parsing, deterministic export.
- `dam-egui`: shared egui app state and UI.
- `apps/native`: thin desktop launcher and native I/O adapters.
- `apps/web`: thin WASM launcher and web/static asset adapters.

## Platform Targets

Target both:

- Native desktop
- Browser/WASM

Use one shared egui UI with small platform adapters.

Use `eframe` + `egui` for native and WASM.

Use Trunk for web development/build:

- `trunk serve`
- `trunk build`

## Theme

Dark theme only.

No light/dark theme switcher in the first version.

## Map Preview

Use `walkers` inside a docked/resizable panel in the main creation screen.

The preview should be part of the workflow, not a floating `egui::Window`.

Use:

- simplest dark map background
- offline/local PMTiles support
- plain dark fallback if PMTiles is unavailable

PMTiles must not be embedded in the binary/WASM. Load it as a runtime/static web
asset:

- native: from `assets/pmtiles/...`
- web: from Trunk static output

The map preview is non-authoritative for export. It is used to confirm selected
static map geometry and later becomes the foundation for dynamic map drawing.

No separate `Preview` button is needed in the first version. The docked preview
updates live when the selected map changes.

## Map Catalog

Static maps come from GeoJSON files in a specific folder.

Example:

```json
{
  "type": "FeatureCollection",
  "name": "HAUT VALAIS",
  "description": "MAP, WARNINGS, TRA, 50714 - HAUT VALAIS",
  "features": [
    {
      "geometry": {
        "type": "LineString",
        "coordinates": [
          [7.8886111111111115, 46.48583333333333],
          [8.081666666666667, 46.26138888888889]
        ]
      },
      "type": "Feature",
      "properties": {
        "priority": 18,
        "color": "#2d5f5f",
        "linePattern": "PS_USERSTYLE 0505",
        "tooltipInfo": "LS-T 24 HAUT VALAIS\\WITHOUT BUFFER"
      }
    }
  ]
}
```

Catalog rules:

- Map id comes from the filename stem, for example `50714.geojson` -> `50714`.
- Map name comes from top-level `name`.
- Description comes from top-level `description`.
- Feature properties are preserved for preview/catalog use.
- Duplicate ids are assumed impossible.
- Duplicate names are allowed; id is authoritative.
- Map selector label is `id - name`.
- Description is secondary/search metadata.
- Search is required from the first version:
  - case-insensitive substring search
  - search id, name, and description
  - sort by id/name
- First version supports one predefined static map only.
- Map groups are deferred.

Bundling:

- Bundle GeoJSON maps with `include_dir` initially.
- Keep the catalog abstraction ready for native filesystem loading later.
- First version ships bundled maps only.
- Invalid GeoJSON files should be skipped and reported in a collapsible
  diagnostics panel.
- Block export only when no valid maps are available or no valid map is selected.

## Static vs Dynamic Maps

First version supports static predefined maps only.

Dynamic/free-drawn maps are deferred.

For static maps:

- Selecting a static map sets the DAM map name to the static map name.
- The DAM map name is shown as read-only.
- The operator cannot edit the DAM map name for static maps.
- Export includes only map id and map name.
- GeoJSON is not included in the export for static maps.

Later dynamic maps:

- Operator must choose a name manually.
- Geometry drawn by the operator will later be converted to AIXM.

## Dates And Times

No timezone semantics.

Dates and times are plain operational values:

- entered exactly as displayed
- validated as plain date/time values
- exported exactly as entered
- no UTC labels
- no Swiss-local conversion
- no timezone-aware model

Use `chrono` for typed `NaiveDate` and `NaiveTime` internally.

Serialize at the export boundary as deterministic strings:

- dates: `YYYY-MM-DD`
- times: `HH:MM`

## Date Range And Weekdays

The form has start date and end date.

Default:

- start date = current date
- end date = current date

Rules:

- End date must be on or after start date.
- If end date differs from start date, show weekday checkboxes.
- If date range is same-day, hide weekday checkboxes.
- Weekday selection defaults to possible weekdays included in the selected range.
- Operators may deselect active weekdays.
- If date range changes, update possible weekdays while preserving valid user
  deselections where still applicable.

Export represents repetition as:

- date range
- active weekdays
- period rows

Do not expand into individual date instances in the first version.

## Periods

Multiple period rows are allowed.

Each period row includes:

- Start indication flag
- Start time
- End indication flag
- End time
- Low level value + effective unit
- High level value + effective unit

Defaults:

- Start indication enabled
- End indication enabled
- low level `000`
- high level `999`

Validation:

- At least one period is required.
- Time format is `HH:MM`.
- `start_time < end_time`.
- Identical start/end times are invalid.
- Overnight periods are invalid in the first version.
- Overlapping periods are invalid for the same active day.
- Adjacent periods are valid.

Keyboard shortcuts from the PDF:

- When focus is in Start time, pressing `N` fills current system time as `HH:MM`
  and moves focus to End time.
- When focus is in End time, pressing `E` fills `23:59`.
- Shortcuts only apply while focus is inside the relevant field.

Add/Remove behavior:

- Add period inserts a new row below the current/focused row.
- Remove period removes the current/focused row.
- At least one row must remain.

## Altitude Levels

Use explicit compact unit selectors: `FL | ft`.

The old hidden 4-digit-implies-feet behavior becomes visible and deterministic:

- Each level field stores the user's last explicit unit choice as UI state.
- If the value reaches 4 digits, effective unit is forced to `ft`.
- The unit selector shows `ft` and is disabled while 4 digits are present.
- When the value returns to 3 digits or fewer, the selector is enabled again and
  restores the last explicit unit choice for that field.
- This applies independently to low and high levels.

Export stores only:

- entered value
- effective unit

Do not export normalized feet values.

Ordering validation:

- If both levels are `FL`, require `low <= high`.
- If both levels are `ft`, require `low <= high`.
- If one level is `FL` and one is `ft`, convert FL to feet using
  `FL n = n * 100 ft`, then require `low <= high`.

## Corrections And Buffer Filters

Use three segmented selectors.

Altitude correction:

- `None`
- `QNH Corr`
- `FL Corr`

Rules:

- `QNH Corr` and `FL Corr` are mutually exclusive.

Upper buffer:

- `Default`
- `UL half`
- `UL no buffer`

Rules:

- `UL half` and `UL no buffer` are mutually exclusive.

Lower buffer:

- `Default`
- `LL half`
- `LL no buffer`

Rules:

- `LL half` and `LL no buffer` are mutually exclusive.

The three categories are independent of each other.

No `Alps` option in the first app.

## Unit/Sector Distribution

Implement Unit/Sector selection from the PDF now, but keep it as metadata in the
export.

Use a modal/panel with grouped checkboxes.

Rules:

- At least one sector must be selected.
- Selecting a unit selects all related sectors.
- Deselecting a unit deselects all related sectors.
- If all sectors in a unit are selected, the unit becomes selected.
- If the last selected sector in a unit is deselected, the unit becomes
  deselected.
- Upper and lower/app/FIC/SPVR/FMP groupings should follow the PDF.

No actual CWP distribution in the first version.

## Text And DABS Info

Implement:

- editable `Text`
- `Display Text` flag

Rules:

- Text limit: 250 characters.
- `Display Text` defaults off.

DABS Info:

- read-only
- empty for now
- included only if useful for schema compatibility

DABS map creation is out of scope.

## Send / Export

`Send` in the first version means:

- validate the form
- build a typed Rust `DamCreation` model
- show errors inline if invalid
- generate deterministic pretty JSON
- export the JSON locally

Native:

- save JSON to a file.

WASM:

- trigger browser download.

No real SKYVISU sending in the first version.

AIXM:

- final target format is AIXM
- AIXM template will be provided later
- AIXM renderer should consume the same stable Rust domain model

Interim JSON:

- pretty formatted
- deterministic field order where practical
- stable field names
- no UI-only state
- static map exports only id and name

## Reset / Cancel

Include a `Reset` action.

In a standalone app, this replaces the PDF's `Cancel` behavior.

If the form is dirty, ask for confirmation before resetting.

## Diagnostics

Add a collapsible developer diagnostics panel.

Collapsed by default.

Use it for:

- map catalog parse warnings/errors
- missing PMTiles fallback status
- validation summary
- build/version info

No settings screen in the first version.

## Validation Summary

Strict validation is required for known DAM/export constraints:

- selected valid static map
- read-only DAM map name derived from selected static map
- valid date range
- valid active weekdays for multi-day ranges
- at least one period
- valid time formats
- start time before end time
- no overnight periods
- no overlapping periods
- valid altitude values
- altitude ordering after FL-to-feet comparison
- correction/buffer selector exclusivity
- at least one Unit/Sector selected
- text length <= 250

Do not add AIXM-specific validation until the AIXM template exists.

## Testing Baseline

Before considering migration complete:

- `cargo test`
- native compile/smoke build
- web `trunk build`

`dam-core` unit tests should cover:

- validation
- add/remove period model behavior
- date range and weekday selection behavior
- level unit/effective-unit logic
- altitude ordering with mixed units
- map catalog parsing from sample GeoJSON
- deterministic JSON export

UI tests are not required initially beyond compile/smoke builds.

## Deferred Work

Later slices:

- AIXM export using provided template.
- Real SKYVISU adapter if protocol/API is provided.
- Dynamic/free-drawn maps.
- Converting drawn geometry to AIXM.
- Map groups.
- Native external maps folder loading.
- More complete PMTiles/source configuration.
- Full DAM management lifecycle.
- Active/Today/Repetitive map lists.
- CWP overlap/transparency/display behavior.
- Installers, signing, and distribution packaging.
