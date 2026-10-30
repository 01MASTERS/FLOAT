---
last_mapped_commit: b0aa3e18784a67a4925e2a1ba9995da723742ca0
last_mapped_at: 2026-10-30
---
# Codebase Concerns

**Analysis Date:** 2026-10-30

## Tech Debt

**Fixed Window Size & Hit-Testing:**
- Issue: The application window is fixed at 720×400 logical pixels, relying on dynamic cursor testing (`set_ignore_cursor_events`) to allow click-through.
- Files: `src-tauri/src/window.rs`, `src/components/float/FloatShell.tsx`
- Impact: If the expanded surface UI grows beyond 720×400px or if multi-monitor coordinate translation has minor DPI discrepancies, clicks may be blocked or pass through unexpectedly.
- Fix approach: Keep surface layouts strictly constrained within the bounding region or implement robust DPI-aware monitor boundary tracking.

**State Storage Fragmentation:**
- Issue: Settings and tasks are stored across both browser `localStorage` and native filesystem JSON (`shelf.json`, `window-position.json`).
- Files: `src/services/settings.ts`, `src/activities/tasksStore.ts`, `src-tauri/src/shelf.rs`, `src-tauri/src/window.rs`
- Impact: Clearing browser cookies or cache could reset frontend settings, while filesystem files persist.
- Fix approach: Eventually unify persistent storage into a single local JSON store managed cleanly via Tauri IPC.

## Known Bugs & Edge Cases

**Multi-Monitor DPI Scaling:**
- Symptoms: When dragging between monitors with different DPI scalings (e.g. 100% and 150%), hit-test rectangles or window positioning may temporarily misalign until settled.
- Files: `src-tauri/src/window.rs`, `src/platform/index.ts`
- Trigger: Rapidly dragging the island across screen boundaries.
- Workaround: The native window position resets and recalculates monitor metrics on drag settlement.

## Security Considerations

**Private Calendar Feed URLs:**
- Risk: User's private iCal link (which may contain sensitive calendar event titles and attendee data) is stored in the application.
- Files: `src/services/settings.ts`, `src-tauri/src/calendar.rs`
- Current mitigation: URLs and event data are never transmitted to third-party cloud servers; they are only stored locally and requested directly from the provider.
- Recommendations: Ensure URLs are never printed in debug logs or committed to repositories.

## Performance Bottlenecks

**Background Hit-Test Polling:**
- Problem: The native background thread tests the mouse cursor against hit regions at 60Hz (16ms tick).
- Files: `src-tauri/src/window.rs`
- Cause: Ensuring low-latency click-through response when the mouse enters or leaves the dynamic island.
- Improvement path: Optimize mouse coordinate testing logic with simple bounding-box rejection before detailed polygon testing.

## Fragile Areas

**Windows GSMTC Session Tracking:**
- Files: `src-tauri/src/media.rs`
- Why fragile: Windows media transport sessions can rapidly cycle, report stale playback states, or fail to yield album art streams if apps (e.g. Spotify, Chrome) lock the handle.
- Safe modification: Maintain defensive timeout wrappers and fallback placeholders when album art streams fail.

## Test Coverage Gaps

**Frontend Automated Unit Tests:**
- What's not tested: React components and activities store state transitions lack automated Jest/Vitest unit tests.
- Files: `src/components/float/*.tsx`, `src/activities/*.ts`
- Risk: Regressions in activity priority ordering or tab transitions could occur without compiler detection.
- Priority: Medium.

---

*Concerns audit: 2026-10-30*
