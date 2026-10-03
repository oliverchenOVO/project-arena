# ARENA design system

Reference: `race-concept.png` (design reference; its grid illustration is never used as execution data).

Colors: near-black #0b0e13, graphite #141922, border #28313e, text #d9e4ef, muted #8d9caf, Dijkstra #569ee7, A* #4dd6c6. Minimal 2px radius. Segoe UI/system sans-serif chrome; Consolas/ui-monospace values. 8px spacing rhythm, 24px screen gutters, 240px configuration rail, equal canvas panes, shared full-width playback and legend.

Primary composition: header ARENA + BUILD/RACE/INSPECT/ANALYZE + import/export; heading Pathfinding arena, subtitle Same problem. Same rules. Different search.; Run experiment action; config rail; two race panes; shared timeline. Four contenders use the same viewport family. BUILD uses one larger canvas and a terrain toolbar. INSPECT uses the shared race panes plus node details. ANALYZE uses a comparison table and measured/derived/hypothesis sections. On narrow screens the rail collapses into a top configuration band and panes stack. Required functional adaptations preserve the palette, geometry and typography.

Canvas colors encode terrain and event state; labels expose start/goal and terrain costs. Current expansion is amber, final path white, frontier outlined in algorithm accent, expanded filled in a darker accent. No raster assets in the renderer. Motion is requestAnimationFrame-driven playback of real events.
