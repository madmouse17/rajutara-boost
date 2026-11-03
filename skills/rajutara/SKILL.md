---
name: rajutara
description: Use when building on or changing rajutara (apparel CAD in Rust: Style/Piece/Marker/CutPlan model, DXF/RUL/HPGL/PDF I/O, nesting, grading, cut plan, egui workspace modules).
---

# rajutara

Read `guidelines/rajutara.md` first (invariants: mm `f64`, no panic on user data, determinism, clean room), then only the guideline that matches the task:

- `guidelines/core-model.md` — Style, Piece, Contour, Notch, SizeTable, GradeRule, Marker, CutPlan, validation, undo.
- `guidelines/io.md` — every loader and writer of `rajutara-io`.
- `guidelines/nesting.md` — `nest_marker`, options, determinism, plotting a marker.
- `guidelines/grading.md` — deriving sizes, editing rules, RUL export.
- `guidelines/cutplan.md` — Orders, lay solver, results, reports.
- `guidelines/ui-modules.md` — adding a workspace `Module`.
- `guidelines/testing.md` — gates, kittest, insta, proptest.
- `guidelines/style.md` — code rules and process.

Runnable references: `examples/`. Every signature in the guidelines is checked against a checkout with `scripts/verify-signatures.py`; this pack matches rajutara 1.5.0 (`CHANGELOG.md`).
