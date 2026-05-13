# DAM Creation Tool — Features

A Rust + egui application for creating Dynamic Airspace Maps (DAMs). V1 is a
creation-only tool: the generated and downloaded legacy SKYVISU-compatible AIXM
XML file is the product.

## Targets and Architecture

- Native desktop launcher (`apps/native`) and Web/WASM launcher (`apps/web`).
- `dam-core` owns the domain model, validation, catalog parsing, geometry
  helpers, and AIXM export.
- `dam-egui` owns UI state, preview rendering, XML preview/editing, and local
  download integration.
- UI state is parsed into `DamCreation`; core validates and exports AIXM; UI
  downloads the resulting XML.

## Application Layout

- Top toolbar actions: `Preview AIXM`, `Download AIXM`, and `Reset`.
- Resizable left form panel with sections: **Map**, **Validity**,
  **Today/Repetitive Periods**, **Corrections & Buffers**, **Distribution**,
  **Additional Information**, and **Diagnostics**.
- Central map preview panel powered by `walkers`.
- Floating overlays: AIXM preview/editor, date picker, and a blocking reset
  confirmation modal.

## Map Selection

Two modes are available: **Predefined map** and **Manual map**.

### Predefined Static Maps

- Catalog is built at compile time from `assets/maps/*.geojson`.
- Map id comes from the filename stem; name comes from GeoJSON top-level
  `name`; description comes from `description`.
- Search is UI-only and filters by id, name, and description. Search changes do
  not invalidate an edited XML draft.
- Selecting a map applies catalog defaults where available: levels, indications,
  display levels, corrections, buffers, additional information, label position,
  and legacy distribution station defaults.
- AIXM export uses the selected catalog `mapId` and uppercase catalog name.

### Manual Maps

- Manual names are forced uppercase while typing and uppercased defensively at
  export.
- Manual geometries:
  - **Polygon** — up to 10 point/arc rows, exported as repeated `gml:pos` and
    real `gml:ArcByCenterPoint` segments.
  - **Para symbol** — single point, empty geometry segments, display levels
    forced to `NO`, name must contain `PARA`.
  - **Text and number** — point plus text/color/size metadata, empty geometry
    segments, display levels forced to `NO`.
  - **Pie / circle** — center, radius in NM, signed angles in `-360..=360`,
    exported as `gml:ArcByCenterPoint`.
  - **Strip** — two endpoints plus width in NM; export computes four corners.
- Manual attributes:
  - Category exports as uppercase `att`.
  - Lateral buffer is available for Polygon, Pie/Circle, and Strip. A non-zero
    buffer exports two geometry components: original geometry first, buffered
    geometry second.

## Validity and Periods

- Start/end dates use `YYYY-MM-DD`; single-day ranges omit `aixm:day`.
- Repetitive ranges emit one timesheet per selected weekday per activation
  period using `MON` through `SUN`.
- Up to 16 activation periods are supported.
- Period fields: start indication, start time, end indication, end time, low
  level, high level.
- Overnight periods and overlaps on effective weekdays are invalid.
- FL values export without zero padding; FT values export directly.
- `Display levels` is available for predefined maps, manual polygon,
  pie/circle, and strip. It is hidden/forced off for para and text/number.

## Corrections, Buffers, A9

- Altitude correction exports to `mapInformation`:
  - None -> `qnh=0;flc=0`
  - QNH Corr -> `qnh=1;flc=0`
  - FL Corr -> `qnh=0;flc=1`
- Upper buffer exports to `ulh`/`uln`; lower buffer exports to `llh`/`lln`.
- A9 UI options are `140`, `150`, and `160`; they export as `140:0`, `150:1`,
  and `160:2` in `mapInformation lvl=`.
- Source is fixed for V1 as `src=ZRH`.

## Distribution

- V1 uses the legacy 12-position distribution, in this exact order:
  `ACC_UPPER`, `ACC_LOWER`, `APP`, `FIC_DELTA`, `ARFA`, `TWR_ZURICH`,
  `TDI_BERN`, `TDI_BUOCHS`, `TDI_DUBENDORF`, `TDI_EMMEN`, `TDI_LUGANO`,
  `TDI_ST_GALLEN`.
- Defaults are all selected unless catalog station defaults map to a known
  subset.
- Export uses slash-separated `1`/`0` flags in `mapInformation dist=`.
- Empty distribution blocks download.

## Additional Information

- Free text, max 250 characters.
- Trims on export and maps to `mapInformation txt=`.
- Newlines and control characters are rejected before export.

## AIXM Preview and Download

- `Preview AIXM` opens a resizable XML preview/editor.
- Form-generated XML is the baseline. Edited XML drafts are draft-only and do
  not write back into the form.
- `Download AIXM` downloads the edited draft when it is well formed; otherwise
  it downloads regenerated form XML.
- Native builds open an operating-system save dialog so the file location can be
  chosen; web builds use the browser download flow.
- Malformed edited XML blocks download with field `aixm.xml`.
- Any AIXM-affecting form change discards the edited draft and regenerates from
  the form.
- Filename format:
  - Predefined: `DAM-{MAP_ID}-{AIXM_NAME}-{YYYYMMDD}.xml`
  - Manual: `DAM-{AIXM_NAME}-{YYYYMMDD}.xml`

## AIXM Export Mapping

| Form/model field | AIXM mapping |
| --- | --- |
| Predefined map id | `ext:mapId` |
| Predefined/manual name | uppercase `aixm:name` |
| Source | fixed `src=ZRH` |
| A9 | `mapInformation lvl=` |
| Distribution | `mapInformation dist=` 12 flags |
| Altitude correction | `mapInformation qnh=` and `flc=` |
| Upper/lower buffers | `mapInformation ulh/uln/llh/lln` |
| Additional information | `mapInformation txt=` |
| Manual category | `mapInformation att=` |
| Manual geometry type | `mapInformation typ=` |
| Manual geometry metadata | deterministic `mapInformation dinfo=` |
| Text/number text/color/size | `tx`, `tc`, `tf` |
| Activation periods | one or more `aixm:Timesheet` elements |
| Display levels | `ext:displayLevels` |
| Label/point position | `ext:displayPositionLevelIndication` |

Fixed V1 metadata includes `designator=UAV`, `type=OTHER`,
`sequenceNumber=0`, `correctionNumber=0`, `sliceId=0`, and
`mapHasBeenDeleted=NO`.

## Validation

Core validation blocks download for invalid or unsupported visible state,
including:

- no selected predefined map or incomplete manual map;
- missing manual name;
- para name without `PARA`;
- empty distribution;
- invalid coordinates, missing label/display positions where required;
- polygon arcs without adjacent point anchors;
- invalid angles outside `-360..=360`;
- empty, too many, overnight, or overlapping periods;
- invalid level ordering;
- text length/newline/control-character violations;
- malformed edited XML.

## Diagnostics

A collapsible developer panel shows catalog parse diagnostics, tile source
status, and build/version info.

## Out of Scope for V1

- Server delivery or lifecycle integration.
- Modify/delete lifecycle.
- Draft save/load.
- Source-site selector.
- Geometry editing for predefined maps.
- New AIXM schema.

## Source Map

| Area | File |
| --- | --- |
| Domain model and enums | `crates/dam-core/src/model.rs` |
| Validation rules | `crates/dam-core/src/validation.rs` |
| Shared geometry helpers | `crates/dam-core/src/geometry.rs` |
| Map catalog parsing / bundling | `crates/dam-core/src/catalog.rs` |
| Legacy distribution | `crates/dam-core/src/distribution.rs` |
| AIXM export and payloads | `crates/dam-core/src/export/` |
| App state and panels | `crates/dam-egui/src/lib.rs` |
| Form state and click-to-place flow | `crates/dam-egui/src/form.rs` |
| Map overlay rendering | `crates/dam-egui/src/preview.rs` |
