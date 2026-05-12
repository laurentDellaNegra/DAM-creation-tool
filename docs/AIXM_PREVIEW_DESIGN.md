# AIXM Preview Side Panel

## Goal

Provide a right-side AIXM workspace before Send. Operators can inspect the
generated AIXM, explicitly enter XML edit mode, then send or download either the
form-generated XML or the active edited XML draft.

## Source of Truth

The left form remains authoritative for DAM authoring. The XML editor does not
write back into the form, catalog-derived state, or map drawing state.

When the preview opens for the first time, the XML is generated from the current
form. `Edit XML` unlocks the text buffer. From that point, any text change is a
draft for submission/download only.

## Draft Lifecycle

- Closing and reopening the preview keeps the active XML draft.
- `Discard XML changes` regenerates XML from the current form and returns to
  read-only mode.
- Any AIXM-affecting form change automatically discards an edited draft,
  regenerates from the form, and shows `AIXM draft discarded because form
  changed.`
- Predefined-map search/filter text is UI-only and does not invalidate a draft.
- If the form becomes invalid, the edited draft is still discarded and the panel
  shows the validation/export errors from the form state.

## Actions

Read-only footer:

- `Edit XML`
- `Download AIXM`
- `Send`

Edit footer:

- `Discard XML changes`
- `Download AIXM`
- `Send`

Global toolbar `Send` and `Download AIXM` use the active edited XML draft when
one exists, even if the preview panel is closed. Otherwise they generate XML
from the current form.

Malformed edited XML blocks `Send` and `Download AIXM`. The panel disables those
buttons and displays the XML parse error. Global toolbar actions surface the
same error as a validation toast.

## Predefined Map Fallback Data

The `mapId` remains authoritative for downstream systems. The AIXM still carries
one fallback `geometryComponent` for processors that cannot fetch the map by
`mapId`.

- The fallback geometry is the first polygon/ring path parsed from the selected
  GeoJSON map.
- If no polygon/ring path exists, the exporter keeps the hardcoded static
  fallback geometry.
- The fallback label position comes from the first GeoJSON point.
- Multiple GeoJSON geometries are not emitted as multiple AIXM geometry
  components.

GeoJSON point symbols are for map display only. `A_SYMBOL_SYM31` is rendered as
the para symbol in the preview map; unknown symbol codes use a generic marker.
Symbols are not represented as AIXM geometry.
