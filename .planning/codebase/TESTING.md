---
last_mapped_commit: 7e8c63329089a1fd40f6906071099b7166d784fe
last_mapped_at: 2026-10-26
---
# Testing Patterns

**Analysis Date:** 2026-10-26

## Test Framework

**Current Status:**
- **No automated test runner is currently installed or configured** in either the frontend (`package.json`) or backend (`src-tauri/Cargo.toml`).
- Verification is currently conducted via TypeScript type checking (`tsc`) and manual desktop testing (`tauri dev`).

**Run Commands:**

```bash

# Type check and build bundle

npm run build          # Runs "tsc && vite build"

# Rust backend compilation check

cd src-tauri && cargo check

# Run app in dev mode for manual testing

npm run tauri dev
```

## Recommended Test Framework Setup

**Frontend Runner (Proposed):**
- Framework: Vitest (`vitest`) + React Testing Library (`@testing-library/react`)
- Mocking: `vitest` built-in mocking for `@tauri-apps/api`
- Proposed Config: `vitest.config.ts`

**Backend Runner (Proposed):**
- Standard Rust test harness (`cargo test`) for pure logic modules (e.g. `calendar.rs`, `rrule` parsing, `shape` calculations).

## Test File Organization (Proposed)

**Location:**
- Co-located next to target implementation files (`src/activities/__tests__/` or `src/island/shape.test.ts`).

**Naming:**
- `*.test.ts`, `*.test.tsx`, `*.spec.ts`

## Mocking Requirements for Platform Abstraction

Because FLOAT relies on native desktop IPC, automated testing of React components requires mocking the Platform Abstraction Layer (`src/platform/index.ts`).

**Mocking Pattern Example:**

```typescript
import { vi } from "vitest";

// Mocking platform calls for UI testing
vi.mock("../platform", () => ({
  setHitRegions: vi.fn(),
  startWindowDrag: vi.fn(),
  getIslandAttached: vi.fn().mockResolvedValue(true),
  subscribeToIslandHover: vi.fn((cb) => {
    cb(false);
    return () => {};
  }),
  subscribeToMultiSessionState: vi.fn(() => () => {}),
  subscribeToNotificationPresence: vi.fn(() => () => {}),
  subscribeToVolumeChanged: vi.fn(() => () => {}),
  subscribeToPrivacy: vi.fn(() => () => {}),
  subscribeToPower: vi.fn(() => () => {}),
}));
```

**What to Mock:**
- Tauri IPC handlers (`invoke`, `listen`).
- Windows DWM compositor interfaces and WinRT objects.
- WASAPI audio streams (`cpal`).
- Browser `localStorage` during component isolation tests.

**What NOT to Mock:**
- Mathematical geometry logic in `src/island/shape.ts`.
- Pure business logic and priority queues in `src/activities/useActivities.ts` and `src/activities/types.ts`.
- Local task store operations in `src/activities/tasksStore.ts`.
- Settings parsing and validation rules in `src/services/settings.ts`.

## Coverage

**Requirements:**
- None enforced currently.
- Recommended targets once tests are introduced:
  - 85%+ coverage on `src/activities/` (priority scheduling, timer calculations).
  - 90%+ coverage on `src/island/shape.ts` (path generation, corner smoothing).
  - 80%+ coverage on `src/services/settings.ts` (storage serialization, schema migration).

## Test Types Needed

**Unit Tests:**
- Activity Priority sorting in `src/activities/useActivities.ts` (verifying Volume HUD preempts notifications, notifications preempt media).
- Timer arithmetic and drift prevention in `src/activities/timerStore.ts`.
- Apple corner smoothing and SVG path string generation in `src/island/shape.ts`.
- Settings default merging and version upgrades in `src/services/settings.ts`.

**Integration Tests:**
- Component interactions between `FloatPill.tsx`, `SplitBubble.tsx`, and `FloatSurface.tsx`.
- Tab navigation and state retention in `FloatSurface.tsx`.

**E2E Tests:**
- Desktop automated testing with Playwright or WebDriver for Tauri applications.

---

*Testing analysis: 2026-10-26*
