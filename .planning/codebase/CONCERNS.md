---
last_mapped_commit: 7e8c63329089a1fd40f6906071099b7166d784fe
last_mapped_at: 2026-10-26
---
# Codebase Concerns

**Analysis Date:** 2026-10-26

## Tech Debt

**Monolithic Orchestrator Component (`FloatShell.tsx`):**
- Issue: `src/components/float/FloatShell.tsx` is over 1,325 lines (~50KB) and manages an enormous surface area of responsibilities: visual mode state machine, gesture handling (wheel volume, wheel skip), dwell timers, drag settlements, keyboard focus, backdrop shapes, hit test rectangle generation, and activity subscriptions.
- Files: `src/components/float/FloatShell.tsx`
- Impact: High cognitive complexity; hard to modify mode transitions or add new gesture interactions without unintended side effects.
- Fix approach: Extract concerns into modular hooks: `useIslandGeometry()`, `useIslandGestures()`, `useWindowHitRegions()`, and `useIslandDrag()`.

**Large Settings View Component (`FloatSettingsView.tsx`):**
- Issue: `src/components/float/FloatSettingsView.tsx` is ~800 lines (~31KB), packing all settings categories, sliders, segmented pickers, and help text into one monolithic file.
- Files: `src/components/float/FloatSettingsView.tsx`
- Impact: Difficult to maintain and review as new configuration toggles are added.
- Fix approach: Break into category components (e.g., `GeneralSettings.tsx`, `AppearanceSettings.tsx`, `PrivacySettings.tsx`, `IntegrationSettings.tsx`).

## Fragile Areas

**Cursor Hit-Testing Synchronization:**
- Files: `src/components/float/FloatShell.tsx` and `src-tauri/src/window.rs`
- Why fragile: The transparent window remains click-through via a 16ms native polling loop that compares cursor position against `HitRect` boundaries sent from the React UI. During rapid Framer Motion morphing (e.g. notch expanding on hover or shrinking to orb), there is a brief latency window between CSS/SVG bounding box changes and the native thread updating `state.regions`.
- Safe modification: Ensure `MORPH_SETTLE_MS` and hit region padding (`HIT_PADDING = 4`) remain generous enough to encompass active spring transitions.
- Test coverage: No automated tests; currently relies entirely on manual verification.

**Hardcoded Window Geometry Synchronization:**
- Files: `src/components/float/FloatShell.tsx` (`WINDOW_WIDTH = 720`, `SURFACE_WIDTH = 580`, `SURFACE_HEIGHT = 244`) and `src-tauri/src/window.rs` (`WINDOW_WIDTH = 720.0`, `WINDOW_HEIGHT = 400.0`)
- Why fragile: The native window size is fixed at 720×400 logical pixels. If the expanded panel or split bubble exceeds these dimensions, it will be clipped without native window scrollbars.
- Safe modification: When expanding panel dimensions or adding wider split bubbles, update constants in both files simultaneously.

**DPI Scaling and Multi-Monitor Coordinate Mapping:**
- Files: `src-tauri/src/window.rs` and `src-tauri/src/backdrop.rs`
- Why fragile: Converting between logical pixels in WebView2 and physical pixels in Win32 APIs (`GetDpiForWindow / 96.0`) can introduce 1-pixel rounding discrepancies when windows straddle different monitor scaling configurations (e.g. 150% laptop screen vs. 100% external monitor).
- Safe modification: Test multi-monitor boundary crossings thoroughly when adjusting window positioning math.

## Security Considerations

**Calendar Subscription Token Storage:**
- Risk: Private iCal / Webcal URLs often embed query string authorization tokens (e.g., Google Calendar or Outlook secret feed URLs).
- Files: `src/services/settings.ts` (`calendarUrl`)
- Current mitigation: Stored in the local browser profile `localStorage` under `float_settings_v1`.
- Recommendations: Consider storing sensitive credentials or URL tokens via Windows Credential Manager (`Windows.Security.Credentials.PasswordVault`) using a dedicated Tauri command.

**Clipboard History Sensitivity:**
- Risk: Unintended capture of sensitive copied strings (e.g., recovery phrases, API keys).
- Files: `src-tauri/src/clipboard.rs`
- Current mitigation: Strictly in-memory (never written to disk), filtered against Windows exclusion flags (`CanIncludeInClipboardHistory = 0` and `ExcludeClipboardContentFromMonitorProcessing`), and capped at 24 entries.
- Recommendations: Provide a quick UI pause toggle directly in the Tray / Clipboard view and consider automatic timeout purges.

**Webview Content Security Policy (CSP):**
- Risk: `src-tauri/tauri.conf.json` sets `"csp": null`.
- Files: `src-tauri/tauri.conf.json`
- Current mitigation: App runs local bundled assets without external script execution.
- Recommendations: Define a strict CSP restricting script sources to `'self'`.

## Performance Bottlenecks

**Active Visualizer Audio Capture:**
- Problem: Live audio visualization processes WASAPI loopback streams and emits IPC events at ~30Hz.
- Files: `src-tauri/src/visualizer.rs`
- Cause: Continuous RMS calculations across 3 frequency bands and frequent Tauri IPC event dispatches (`audio-levels`).
- Current mitigation: Capture only runs when `set_visualizer_active(true)` is explicitly toggled by the frontend when an equalizer is visible on screen.
- Improvement path: Throttle emission frequency on battery power or lower refresh rate when minimized.

**Hardware & Privacy Registry Polling:**
- Problem: Background thread polls `HKCU\Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore` for camera and microphone usage once per second.
- Files: `src-tauri/src/sysmon.rs`
- Improvement path: Leverage Windows registry notification events (`RegNotifyChangeKeyValue`) rather than polling where possible.

## Scaling Limits

**File Shelf & Clipboard In-Memory Limits:**
- Current capacity: File shelf capped at 48 items (`MAX_ITEMS = 48` in `src-tauri/src/shelf.rs`); clipboard history capped at 24 items (`MAX_ENTRIES = 24` in `src-tauri/src/clipboard.rs`) with max text length 20,000 characters and max image size 32MB.
- Limit: Memory usage is bounded by design.
- Scaling path: Maintain existing caps; ensure old items are promptly dropped.

## Dependencies at Risk

**Platform Tight-Coupling (Windows Only):**
- Risk: Heavy reliance on WinRT and Win32 crates (`win-gsmtc`, `wmi`, `windows` 0.48, `cpal`) makes cross-platform porting to macOS or Linux a complete rewrite of the native backend.
- Impact: FLOAT is currently Windows-only.
- Migration plan: Maintain the clean Platform Abstraction Layer (`src/platform/index.ts`) so future OS backends only need to fulfill the existing TypeScript IPC contract.

## Test Coverage Gaps

**Zero Automated Test Coverage:**
- What's not tested:
  - Activity priority sorting and preemption in `src/activities/useActivities.ts`.
  - Continuous corner Bézier math in `src/island/shape.ts`.
  - Settings parsing and version upgrade migrations in `src/services/settings.ts`.
  - Timer wall-clock drift prevention in `src/activities/timerStore.ts`.
- Files: All source files across `src/` and `src-tauri/src/`.
- Risk: High risk of regressions during UI refactoring or feature additions.
- Priority: High.

---

*Concerns audit: 2026-10-26*
