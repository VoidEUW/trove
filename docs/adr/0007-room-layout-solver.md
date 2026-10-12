# 7. Rooms attach to each other's walls, and one solver in the core places them

- Status: Accepted
- Date: 2026-10-12
- Origin: specs "Data model" (room placement) and "3D floor plan with Threlte" in the first repository
- Issue: #1

## Context

A home is drawn as rooms with real measurements. In real flats rooms share walls, and typing
absolute coordinates for every room is error-prone. The 2D editor, the 3D scene and search
highlighting must all agree on where a room is. Decided 2026-10-12: the attachment mechanic is built
in the first spaces milestone, not deferred. Depends on [5](0005-domain-model.md).

## Decision

- A room is either `Free { x, y }` or `Attached { anchor, side: N|E|S|W, offset, door }` to another
  room on the same floor.
- The **layout solver** in `trove-core` is the only producer of absolute positions. It is pure and
  deterministic: same input, same output, on every device.
- The solver refuses attachment cycles and rooms anchored on another floor, and reports overlaps as
  warnings, not errors.
- Staircases are positioned relative to their lower room; the opening on the floor above is derived
  by the solver.
- The 2D editor and the 3D scene only draw solver output; neither computes positions itself.

This rules out absolute-only room positions, and layout maths in the UI.

## Consequences

- The hardest algorithm lands early, before the 3D map, so the map is built on stable positions.
- Resizing a room moves everything attached to it, which is the point but needs clear UI feedback.
- A sync merge that changes an anchor triggers a re-solve; the solver's determinism keeps devices
  equal.
- Follow-ups: solver with property tests (no overlaps for valid input, cycle refusal), layout
  editor, stair-opening derivation.
