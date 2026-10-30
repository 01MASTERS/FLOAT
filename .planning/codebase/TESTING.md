---
last_mapped_commit: b0aa3e18784a67a4925e2a1ba9995da723742ca0
last_mapped_at: 2026-10-30
---
# Testing Patterns

**Analysis Date:** 2026-10-30

## Test Framework

**Runner & Tooling:**
- Frontend: TypeScript type-checking via `tsc --noEmit` / `npm run build`. No automated browser runner (Jest/Vitest) is currently configured in `package.json`.
- Backend: Rust standard testing harness (`cargo test`).
- Native Integration: Tested interactively via manual testing in Tauri runtime and debug logging.

**Run Commands:**

```bash
npm run build          # Typecheck TypeScript and bundle frontend with Vite
cargo check            # Fast syntax & type check for native Rust backend
cargo test             # Run Rust internal unit tests
npm run tauri dev      # Launch live interactive application in development mode
```

## Test File Organization

**Location:**
- Unit tests in Rust are placed inline in their respective modules or in a dedicated `tests/` directory:
  - Example: `src-tauri/src/calendar.rs` contains module-level tests for RRULE expansion and iCal parsing.

**Naming:**
- Rust test modules: `#[cfg(test)] mod tests { ... }`.
- Test functions: `#[test] fn test_something() { ... }`.

## Test Structure

**Rust Test Pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_rrule_parsing() {
        let ics_data = "BEGIN:VCALENDAR...";
        let events = parse_ics(ics_data).expect("Should parse");
        assert!(!events.is_empty());
    }
}
```

## Mocking & Isolation

**Frontend Isolation:**
- `src/platform/index.ts` gracefully degrades when invoked outside Tauri:
  ```typescript
  export async function getIslandAttached(): Promise<boolean> {
    try {
      return await invoke<boolean>("get_island_attached");
    } catch {
      return true; // Safe fallback for headless or browser preview
    }
  }
  ```
- This allows individual React components to be tested or previewed in standard Vite dev servers without crashing on missing native bindings.

## Coverage

**Requirements:**
- No formal coverage threshold enforced at this stage.
- Compilation and strict TypeScript validation (`tsc`) must pass with 0 errors before committing.

## Test Types

**Static Verification:**
- TypeScript strict checking: Ensures interfaces match IPC contracts between frontend and backend.
- Rust borrow checker and type verification: Ensures thread safety across background OS monitors.

**Interactive Verification:**
- User Acceptance Testing (UAT) performed interactively against live Windows DWM, GSMTC media sessions, and system notifications.

---

*Testing analysis: 2026-10-30*
