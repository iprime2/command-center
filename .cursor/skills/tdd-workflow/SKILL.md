---
name: tdd-workflow
description: >-
  Test-driven development for Command Center: failing test first, minimal
  implementation, refactor. Use for new behavior, bug fixes, and any slice that
  changes API or UI contracts.
---

# TDD Workflow

## Cycle

1. **Red** — Write a failing test that expresses the acceptance criterion.
2. Run it; confirm it fails for the right reason.
3. **Green** — Minimal code to pass.
4. Run tests; confirm green.
5. **Refactor** — Clean up without changing behavior; tests stay green.

## Bug fixes

Write a failing reproduction test **before** the fix.

## What to test

- Outcomes and contracts (status codes, JSON shapes, visible UI behavior).
- Not internal call sequences or private implementation details.
- Prefer real code / test DB fakes over heavy mocks except at slow boundaries.

## Anti-rationalization

| Excuse | Rebuttal |
|--------|----------|
| “Too simple to test” | Then the test is five lines — still write it |
| “I’ll test after it works” | That’s not TDD; start red |
| “Coverage % is enough” | Cover the behavior you are changing |

## Exit

- [ ] New/changed behavior has a test that failed before the fix/feature
- [ ] Relevant suite green
