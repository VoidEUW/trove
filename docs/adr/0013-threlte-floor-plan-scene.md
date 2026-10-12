# 13. One Threlte scene draws the floor plan in both 2D and 3D

- Status: Accepted
- Date: 2026-10-11
- Origin: spec "3D floor plan with Threlte" in the first repository
- Issue: #1

## Context

The map is the main screen: rooms, furniture, items as pins, search hits lit up, several floors.
Positions come only from the solver ([7](0007-room-layout-solver.md)). Two renderers, one for 2D and
one for 3D, would drift apart.

## Decision

- One Threlte scene. An orthographic top-down camera is the 2D view; a perspective orbit camera is
  the 3D view. The 2D layout editor uses the same scene with the orthographic camera.
- Room floors as planes with labels, low walls with door gaps, places as boxes (instanced where
  possible), loose items as pins, staircases as stepped boxes linking floors. All floors can be
  shown stacked with a gap.
- Picking by raycast through Threlte's interactivity plugin.
- Pin and box states take their colours from the tokens: selected = brass, found = laurel with a
  pulse, needs attention = danger, not a match = dimmed.
- Budget: 60 fps with 500 places on an M1 MacBook Air.

This rules out a separate SVG or canvas 2D renderer and CSS-tilted fake 3D.

## Consequences

- One code path for 2D, 3D and the editor.
- WebGL is required; the list view is the fallback.
- Still open: walls fixed low vs. cut away by camera angle; labels as HTML overlays vs. SDF text
  (troika).
- Follow-ups: scene skeleton from solver output, camera switch, pin component, highlight states.
