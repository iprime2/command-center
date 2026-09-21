---
name: code-review-quality
description: >-
  Five-axis code review for Command Center (correctness, readability,
  architecture, security, performance) with Must-fix/Should-fix/Nit. Use after
  implementation before QA, or when reviewing agent/human diffs.
---

# Code Review Quality

Approve when the change **improves** the codebase and meets criteria — not only if you would have typed the same code.

## Process

1. Understand AC and intended slice scope.
2. Review **tests first** — do they assert the behavior?
3. Review implementation on five axes.
4. Categorize findings; lead with correctness/security.
5. Verify verification was actually run.

## Five axes

| Axis | Fail if |
|------|---------|
| Correctness | Spec missed, happy-path-only, tests don’t assert behavior |
| Readability | Unclear names, clever control flow, noisy comments |
| Architecture | New pattern without need; business logic in UI; second backend |
| Security | Unvalidated input, injection, XSS, secrets, missing authz |
| Performance | N+1, unbounded lists, blocking on hot path |

## Severity

- **Must fix** — blocks sign-off
- **Should fix** — fix in this slice if small; else track
- **Nit** — optional

## Anti-patterns

- Rubber-stamping because tests are green
- Burying one Must-fix under twenty nits
- Expanding review into unrelated refactors
