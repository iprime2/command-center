---
name: using-cc-skills
description: >-
  Routes Command Center work to the correct project skills (lead, senior, rust,
  frontend, UI/UX, QA, TDD, verify, security, review, API contract, feature
  replication). Use at the start of non-trivial tasks or when unsure which
  skill applies.
---

# Using Command Center skills

Load this first for non-trivial work, then open only the skills you need.

## Decision tree

```text
Task arrives
  ├── Don't know scope / multi-module feature? → lead-engineer
  ├── Implementing a planned slice?
  │     ├── API / DB / migrations → rust-backend-engineer + tdd-workflow
  │     ├── Web UI wiring → frontend-engineer (+ ui-ux-engineer if UX-heavy)
  │     └── General slice → senior-engineer + tdd-workflow
  ├── Designing screens / states / a11y? → ui-ux-engineer
  ├── Replicate one flow from another app? → feature-recon, then feature-adapt
  │     After that slice is built → feature-parity
  ├── Designing or changing HTTP API? → api-contract-design
  ├── Auth, money, secrets, untrusted input? → security-review
  ├── Bug / weird failure? → systematic steps in tdd-workflow + verification-loop
  ├── Code written? → code-review-quality (five-axis)
  ├── Ready to claim done? → verification-loop then qa-engineer
  └── Tiny typo/config? → skip full loop; still verify
```

## Progressive disclosure

- Do not load all skills into context.
- Prefer `paths`-scoped skills when editing `apps/api` vs `apps/web`.
- Skills are workflows with exit criteria — not reference essays.

## Anti-rationalization

| Excuse | Rebuttal |
|--------|----------|
| “Too small for lead/QA” | If >~100 lines or new behavior, use slices + verify |
| “I’ll verify later” | Later never comes — run verification-loop before done |
| “I wrote it so I can QA it” | Author ≠ QA; spawn independent qa-engineer |
