---
name: verification-loop
description: >-
  Hard gate before claiming Command Center work done: run tests/build, exercise
  user-visible paths, paste evidence. Use whenever about to say done, fixed, or
  ready for review/QA.
---

# Verification Loop

“Seems right” is never an exit criterion.

## Checklist

1. Re-read acceptance criteria.
2. Run the **focused** tests for the change.
3. Run package **build/lint/typecheck** (or `cargo test` / `cargo clippy`) for touched apps.
4. If UI changed: **browser flow** — open app, perform the job, observe result.
5. If API changed: hit health and the affected route (curl/httpie or client) when feasible.
6. Paste command output / observed results into the report.

## Done template

```markdown
## Verification
- AC: all met? yes/no
- Commands: …
- UI steps + result: …
- Residual risks: …
```

## Anti-rationalization

| Excuse | Rebuttal |
|--------|----------|
| “Tests passed earlier” | Re-run after the last edit |
| “Compile succeeded” | Compiling ≠ behavior correct |
| “QA will catch it” | You still verify before handoff |

## Exit

Only claim done when the checklist is complete with evidence.
