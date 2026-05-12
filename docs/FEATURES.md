# DAM Creation Tool — Features

A Rust + egui application for creating Dynamic Airspace Maps (DAMs). One shared
UI codebase compiles to a native desktop app and to a WASM web app via Trunk.
The current version focuses on the creation workflow: it produces a typed,
validated domain model and an AIXM XML payload. SKYVISU delivery and full DAM
lifecycle management are deferred.

## Targets and Architecture

- Native desktop launcher (`apps/native`) — `eframe` window, 1440×900 default.
- Web/WASM launcher (`apps/web`) — Trunk-built canvas, mounts on `#dam-canvas`.
- Shared domain crate `dam-core` — model, validation, catalog, and AIXM export.
  No UI dependencies.
- Shared UI crate `dam-egui` — egui app state, form, map preview overlay.
- Dark theme only.

## Application Layout

A single screen split into:

- Top toolbar with `Send`, `Preview AIXM`, `Download AIXM`, and `Reset`
  actions.
- Resizable left form panel (420–760 px) with collapsible sections.
- Central map preview panel (powered by `walkers`) that updates live.
- Floating windows: Unit/Sector picker, date picker calendar, reset
  confirmation.

Form sections, top to bottom: **Map**, **Validity**, **Today/Repetitive
Periods**, **Corrections & Buffers**, **Distribution**, **Additional
Information**, **Status** (only when there are validation issues), and
**Diagnostics** (collapsed by default).

## Map Selection

Two modes are available: **Predefined map** and **Manual map**.

### Predefined static maps

- Catalog is built at compile time from `assets/maps/*.geojson` via
  `include_dir!` (currently ~198 bundled maps).
- Map id derives from the filename stem (e.g. `50714.geojson` → `50714`); name
  comes from the GeoJSON top-level `name`; description from `description`.
- Search box performs case-insensitive substring matching across id, name, and
  description, with a count of matches.
- Results list shows `id - name` (selected entry highlighted) plus the
  description; clicking selects and re-centers the preview.
- Selected map summary panel shows id, name, and description as read-only.
- Invalid GeoJSON files are skipped and reported via the diagnostics panel.

### Manual maps

Five geometry types, each with its own draft state and validation rules:

- **Polygon** — up to 10 nodes. Nodes can be either points or arcs. Arc nodes
  carry a center coordinate plus a radius in NM and are sampled (24 segments)
  between adjacent point anchors when rendered. Optional label position.
- **Para symbol** — single position point.
- **Text and number** — position point + up to 25-char text, color
  (Red/Green/Blue/Yellow/White), size (Small/Medium/Large).
- **Pie / circle** — center, radius (NM), first and last angle (0–360°),
  optional label. Defaults are `0`/`360` for a full circle.
- **Strip** — two endpoints, width (NM), optional label.

Manual map attributes (shared across geometries):

- **Category** — Danger, Restricted, Glider, CTR, CFZ, TMA, Para, Other.
- **Rendering** — Surface or Line.
- **Lateral buffer (NM)** — only shown for Polygon, Pie/Circle, and Strip.

Map-driven authoring helpers:

- Switching geometry type auto-focuses the first relevant coordinate field.
- Focusing a coordinate or distance field arms the preview to accept a click.
- Clicking the map fills the focused field and advances to the next field in a
  scripted flow (e.g. Strip: Point 1 → Point 2 → Width).
- For radius/width fields the click is converted to a great-circle distance in
  NM (perpendicular distance for Strip width) using a Haversine helper.
- Live ghost preview follows the cursor while a field is armed; an anchor line
  and inline distance label are drawn where it makes sense (next polygon point,
  arc/pie radius, strip endpoints/width).

## Validity (Date Range and Weekdays)

- Start and end date fields plus a `Pick` button that opens a calendar window.
  Calendar supports month navigation, “Today” shortcut, and highlights the
  selected day and the current day.
- Defaults to today / today (single-day creation).
- End date must be on or after start date.
- For multi-day ranges, weekday checkboxes (Mon–Sun) are shown; the set of
  *possible* weekdays is derived from the range, and operator deselections are
  preserved across range changes where still applicable.
- Single-day ranges hide the weekday selector entirely; effective weekday is
  taken from the start date.

## Today/Repetitive Periods

- Up to 16 activation periods, each shown as a row with its own controls.
- Per-row fields: `Start indication` flag, start time, `End indication` flag,
  end time, low level (value + unit), high level (value + unit).
- Periods are addable/removable inline; at least one must remain.
- Times are `HH:MM`. End time must be strictly after start time. Overnight
  periods are rejected. Overlapping periods on the same effective weekdays are
  rejected; adjacent periods are valid.
- Keyboard shortcuts (mirroring the legacy tool):
  - `N` while a Start time field is focused fills the current system time and
    moves focus to the matching End time.
  - `E` while an End time field is focused fills `23:59`.
- A `Display levels` toggle controls whether the selected period’s
  `low/high` label is rendered on the map preview.

### Altitude levels

- Each level field stores a value plus a `FL`/`ft` selector.
- 4-or-more digit values force the unit to `ft`; the selector is disabled and
  shows a `4+ digits -> ft` hint. Returning to ≤3 digits restores the user’s
  previously chosen unit.
- Ordering validation: low ≤ high after converting `FL n` to `n × 100 ft`,
  including mixed-unit comparisons.

## Corrections and Buffers

Three independent segmented selectors:

- **Altitude correction** — None / QNH Corr / FL Corr (mutually exclusive).
- **Upper buffer** — Default / UL half / UL no buffer.
- **Lower buffer** — Default / LL half / LL no buffer.

## Unit/Sector Distribution

- Modal window grouped by region (Geneva, Zurich) and unit
  (ACC upper/lower, APP, MIL/DLT/FIC or ARFA/DLT/FIC, SPVR/FMP).
- 10 unit groups with around 40 sectors total; each sector has a stable id like
  `GVA:UL1` or `ZRH:APW`.
- Tri-state semantics: ticking a unit selects all of its sectors; deselecting a
  unit deselects all; the unit checkbox auto-checks when all its sectors are
  selected and unchecks when the last one is removed.
- At least one sector must be selected. The form panel shows a live count.

## Additional Information

- Free-text comment, max 250 characters, with live character counter.
- `Display Text` toggle (defaults off).
- Read-only `DABS Info` placeholder retained for schema parity (empty in this
  version).

## Map Preview

- Docked, non-floating panel powered by `walkers`.
- Online CARTO Dark Matter raster tiles provide the map background.
- The default empty-state viewport remains centered on Switzerland without a
  country-border GeoJSON overlay.
- For predefined maps: renders the selected map’s GeoJSON paths
  (LineString / MultiLineString / Polygon / MultiPolygon) in accent color.
  GeoJSON point symbols are also rendered on the map only: `A_SYMBOL_SYM31`
  uses the same para glyph as manual para symbols, while unknown symbol codes
  use a generic non-editable marker.
- For manual maps: renders the in-progress geometry, including a ghost preview
  that follows the cursor while a coordinate or distance field is focused.
- Optional level label (`low/high`) is drawn at the map’s label position or
  bbox center while `Display levels` is on.
- Initial center/zoom auto-fits the selected map’s bounding box.

## Reset and Send

- **Reset** opens a confirmation window; on confirm, the form is rebuilt from
  defaults, validation state and status are cleared, and the preview re-centers.
- **Download AIXM** runs the same validation path and exports the first-pass AIXM
  XML payload locally. If an edited AIXM draft exists, the draft XML is used
  after a well-formedness check.
- **Preview AIXM** opens a resizable XML preview/editor. The left form remains
  the source of truth; XML edits are draft-only and never write back into the
  form. Any AIXM-affecting form change discards the edited draft and regenerates
  from the form. Search/filter text is UI-only and does not discard the draft.
- **Send** builds the active AIXM payload, using the edited XML draft when one
  exists, and then reports that no submission endpoint is configured yet.

## Validation

`dam_core::validate_creation` enforces, with structured `field`/`message`
issues:

- Selected map: predefined map id and name present, or, for manual maps:
  - non-empty name and a label position;
  - lateral buffer ≥ 0 (when applicable);
  - polygon: ≥3 points, ≤10 nodes, valid lat/lon for each point and arc center,
    arc radius > 0;
  - para symbol / text-number: required position; text non-empty and ≤25 chars;
  - pie/circle: required center, radius > 0, angles in [0, 360], first ≠ last;
  - strip: required and distinct endpoints, width > 0.
- Date range: end ≥ start; for multi-day ranges, at least one active weekday
  and the active set is a subset of the possible weekdays.
- Periods: at least one; at most 16; end > start; level value ≤ 99 999;
  altitude ordering (with FL→ft normalization); no overlapping periods on the
  effective weekdays.
- Distribution: at least one sector selected.
- Text: ≤250 characters.

Failed validation surfaces in a **Status** section under the form, listing
field paths and human-readable messages, and blocks export.

## AIXM Export

AIXM XML export is generated in `dam-core` from the same domain model:

- Dates and times are emitted from the active form state.
- Predefined maps keep `mapId` authoritative for downstream processing.
- Predefined map fallback AIXM includes one `geometryComponent`: the first
  polygon/ring path parsed from the selected GeoJSON map, or the static
  hardcoded fallback when the selected map has no polygon/ring geometry.
- Predefined map fallback label position comes from the first GeoJSON point used
  for `displayPositionLevelIndication`.
- Manual polygon maps emit the drawn geometry and label position with
  `mapId=0`.
- Edited XML drafts must be well formed before `Send` or `Download AIXM`; the
  preview panel disables those actions while XML is malformed, and global
  toolbar actions show validation errors.

## Diagnostics

A collapsible developer panel surfaces:

- Catalog parse warnings/errors per file.
- Online tile source status.
- Build/version info (`CARGO_PKG_VERSION`).

## Known Out-of-Scope (First Version)

Carried over from the migration plan and not implemented:

- Real SKYVISU submission endpoint.
- AIXM lateral-buffer XML geometry and multi-period export.
- Map groups, Active/Today/Repetitive map management, Modify/Delete lifecycle.
- Draft save/load, settings screen.
- Full CWP rendering, transparency/overlap, operational display rules.
- Native installers, signing, packaging.

## Source Map

| Area | File |
| --- | --- |
| Domain model and enums | `crates/dam-core/src/model.rs` |
| Validation rules | `crates/dam-core/src/validation.rs` |
| Map catalog parsing / bundling | `crates/dam-core/src/catalog.rs` |
| Unit/sector reference data | `crates/dam-core/src/distribution.rs` |
| AIXM export and payloads | `crates/dam-core/src/export/` |
| App state, panels, windows | `crates/dam-egui/src/lib.rs` |
| Form state, click-to-place flow | `crates/dam-egui/src/form.rs` |
| Map overlay rendering | `crates/dam-egui/src/preview.rs` |
| Native / WASM launchers | `apps/native/src/main.rs`, `apps/web/src/lib.rs` |
