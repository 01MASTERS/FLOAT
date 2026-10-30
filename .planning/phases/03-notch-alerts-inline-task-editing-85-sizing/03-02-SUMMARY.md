# Phase 3 Plan 02: 85% Sizing Expansion & Hit-Testing Calibration Execution Summary

**Plan:** `03-02`  
**Phase:** `03-notch-alerts-inline-task-editing-85-sizing`  
**Requirements Covered:** `TASK-02`  
**Execution Date:** 2026-10-30  
**Status:** Complete  

---

## 1. Executive Summary

Plan `03-02` delivered requirement `TASK-02` by implementing a unified 85% proportional interface scale across the entire FLOAT application (docked notch, floating pill, expanded surface, resting orb, and satellite split bubble) while maintaining flush $y=0$ top bezel alignment, crisp sub-pixel vector rendering, calibrated native Windows click-through hit-testing, and synchronized compositor glass backdrop coordinates.

Placing `transform: scale(0.85); transform-origin: top center;` on the `#root` DOM element in `src/index.css` uniformly reduces the entire UI without interference from Framer Motion's inline `scale` animations on `.float-shell`. Native click-through hit boundaries in `FloatShell.tsx` were calibrated to scale all target dimensions by `ROOT_SCALE = 0.85` during morph animations and to measure live bounding client rectangles via `getBoundingClientRect()` with an added 2px boundary buffer (`HIT_BUFFER = 2`) once spring physics settle at 700ms (`MORPH_SETTLE_MS`). In addition, the native Windows DWM glass backdrop blur window coordinates and corner radii passed to `setBackdrop()` were scaled by `0.85`, ensuring pixel-perfect backdrop blur alignment without clipping or background bleeding.

---

## 2. Tasks Completed & Commits

| Task | Description | Files Modified | Commit |
|------|-------------|----------------|--------|
| **Task 1** | Unified 85% Root Scaling & Flush Top Bezel Alignment | `src/index.css` | `8d2d0e9` (`feat(03-02): unified 85% root scaling and flush top bezel alignment`) |
| **Task 2** | Calibrated Native Click-Through Hit-Testing with 2px Buffer & Backdrop Synchronization | `src/components/float/FloatShell.tsx`, `src-tauri/src/backdrop.rs` | `86f4e29` (`feat(03-02): calibrated native click-through hit-testing and backdrop sync`) |

---

## 3. Key Architectural Decisions & Details

- **Unified Root Scaling & Framer Motion Isolation (`src/index.css`)**:
  - Implemented Decision **D-10** by setting `transform: scale(0.85); transform-origin: top center;` on `#root`.
  - Isolating the 0.85 scaling transform on `#root` prevents Framer Motion from overwriting the global scale when animating inline `scale` styles on `.float-shell` during button clicks and press squishes.
  - Chromium renders child SVG paths (`islandPath`), typography, and raster assets at native display DPI, guaranteeing crisp vector edges without blurriness.
- **Flush Top Bezel Docking (`src/components/float/FloatShell.tsx`)**:
  - Satisfied Decision **D-11**: with `transform-origin: top center` (horizontal center $x = 360\text{px}$, vertical top $y = 0\text{px}$), any point at $(x, 0)$ remains strictly at $y' = 0 + (0 - 0) \times 0.85 = 0$.
  - Confirmed that `docked` mode maintains `islandTop = 0` so the docked notch remains pinned flush against the physical top edge of the display without an artificial gap.
- **Calibrated Native Hit-Testing & Live DOM Measurements (`src/components/float/FloatShell.tsx`)**:
  - Satisfied Decision **D-12** by defining `ROOT_SCALE = 0.85` and `HIT_BUFFER = 2`.
  - In `regionsFor()`, calculated scaled hit regions during transitions for both the main island and the detached `.split-bubble` satellite (`BUBBLE_SIZE * ROOT_SCALE`, `BUBBLE_GAP * ROOT_SCALE`).
  - At `MORPH_SETTLE_MS` (700ms), queries `shellRef.current.getBoundingClientRect()` and `bubbleEl.getBoundingClientRect()` to lock in the true viewport coordinates produced by CSS transforms.
  - Added a 2px buffer to the measured bounds (`rect.left - HIT_BUFFER`, `rect.top - HIT_BUFFER`, `rect.width + HIT_BUFFER * 2`, `rect.height + HIT_BUFFER * 2`) to eliminate border-crossing hover jitter and prevent premature click-through dropouts. Clamped top coordinate to non-negative values via `Math.max(0, ...)`.
- **Compositor Glass Backdrop Coordination (`src/components/float/FloatShell.tsx`, `src-tauri/src/backdrop.rs`)**:
  - In `drawFrame()`, scaled backdrop dimensions sent to native `setBackdrop()` by `ROOT_SCALE`: `width = f.width * f.scale * ROOT_SCALE`, `height = f.height * f.scale * ROOT_SCALE`, `y = f.y * ROOT_SCALE`, and corner radii `f.top * f.scale * ROOT_SCALE`, `f.bottom * f.scale * ROOT_SCALE`.
  - Documented the scale coordination contract in `src-tauri/src/backdrop.rs`, ensuring that the native `FloatBackdrop` HWND aligns with sub-pixel precision with the 85% scaled overlay.

---

## 4. Verification Results

- `cargo check --manifest-path src-tauri/Cargo.toml`: **Passed** (Compiled `float v2.0.0` with 0 warnings or errors in 19.38s).
- `npm run build`: **Passed** (TypeScript 5.8.3 strict compilation + Vite production bundle passed cleanly with 0 errors in 5.29s).
- `#root` scale verification: **Passed** (`transform: scale(0.85); transform-origin: top center; width: 100%; height: 100%;` applied in `src/index.css`).
- Hit-testing boundary verification: **Passed** (Calculated regions apply `ROOT_SCALE = 0.85` and `HIT_BUFFER = 2`; settle timer captures live `getBoundingClientRect()` for main shell and split bubble).
- Native backdrop alignment: **Passed** (`drawFrame()` supplies scaled coordinates to `setBackdrop()`; `backdrop.rs` coordinates with main window DPI scale).

---

## 5. Artifacts Created & Modified

- `src/index.css` (modified): Applied 0.85 scale and top-center transform origin on `#root`.
- `src/components/float/FloatShell.tsx` (modified): Updated hit-testing calculations, 2px buffer, live settle measurements, and 0.85 backdrop frame coordinates.
- `src-tauri/src/backdrop.rs` (modified): Documented 0.85 scale coordination contract in module header comment block.
- `.planning/phases/03-notch-alerts-inline-task-editing-85-sizing/03-02-SUMMARY.md` (created): Plan summary documentation.
