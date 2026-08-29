# Backend support matrix

Alizarin ships three bindings over one Rust core (`alizarin-core`). They are
**not** identical surfaces — each exposes the capabilities its deployment target
needs. This is a deliberate, supported contract as of 2.0, not partial work.

| Capability | WASM (browser) | Python (notebooks/ETL) | NAPI (Node ETL/builder) |
|---|:--:|:--:|:--:|
| Instance / viewmodel wrapper | ✓ | ✓ | ✓ |
| Resource registry (`getFull`/`getSummary`) | ✓ | ✓ | ✓ |
| Graph / prebuild export | ✓ | ✓ | ✓ |
| SKOS / RDM parse | ✓ | ✓ | ✓ |
| Extension coerce / render / resolve | ✓ | ✓ | ✓ |
| CSV → business_data | — | ✓ | ✓ |
| `batch_trees_to_tiles` / `tiles_to_trees` | ✓ | ✓ | — |
| `coerce*` value family (standalone) | ✓ | ✓ | — |
| Card-display serialization (standalone fn) | ✓ | ✓ | —¹ |

¹ NAPI renders displays through the **instance-wrapper** (`forJson` /
`forDisplayJson` on `NapiResourceInstanceWrapper`); it simply lacks the
*standalone* `serialize_card` entry point.

## Why the surfaces differ (by design)

- **WASM — browser.** No filesystem/CSV ingestion, so **CSV → business_data is
  intentionally absent**. Everything else (conversion, coercion, card display,
  viewmodels) is present because the browser report/editor path needs it.
- **NAPI — Node ETL / builder.** Its consumer (e.g. starches-builder) ingests
  via **CSV → business_data** and mutates through the **instance-wrapper**
  (`setTileDataForNode`, etc.), then exports. It does **not** call standalone
  `batch_trees_to_tiles`, `tiles_to_trees`, or per-value `coerce*` — verified: no
  NAPI consumer references them. So those are **intentionally not exposed**.
- **Python — notebooks / interactive ETL.** The fullest surface, because
  notebook users need every primitive directly.

## The shared-layer guarantee

The backend-agnostic TypeScript layer (`js/`) works on **both** WASM and NAPI. It
depends only on capabilities present in **all** backends it runs on — the
instance-wrapper, registry, export, and extension APIs (top four rows). It does
**not** call the conversion/coerce/card-display group, so NAPI's leaner surface
does not limit it.

## Addable on demand

The NAPI gaps are **thin wrappers over existing shared core** —
`alizarin_core::batch_convert_trees`, `alizarin_core::type_coercion::*`,
`alizarin_core::card_traversal::serialize_card`. If a future NAPI consumer needs
them, exposing them is a mechanical binding change (mirroring the WASM/Python
wrappers), not new core work. They are omitted today because shipping unused
public surface in a stable release is a liability, not a feature.

Likewise WASM's missing CSV path is a Node/Python concern; it would only be added
to WASM if a browser-side CSV-ingestion use case appeared.

## 2.0 statement

For 2.0, the committed surface is the matrix above. "Three backends" means
**three role-specialised bindings over one core**, each complete for its
deployment — not three identical APIs. Consumers should target the backend
matching their environment (browser → WASM, Node ETL → NAPI, notebooks →
Python) and consult this matrix for capability expectations.
