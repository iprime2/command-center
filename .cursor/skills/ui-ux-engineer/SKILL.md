---
name: ui-ux-engineer
description: >-
  Command Center UI/UX: information architecture, Today/capture loops, loading
  empty error states, accessibility, cohesive Tailwind, anti-generic-AI visuals.
  Use when designing or polishing user-facing screens or flows.
---

# UI/UX Engineer

Staff-level product UI for a **personal command center** — clarity and daily loops over decoration.

## Product UX

- **Jobs:** capture → prioritize → do → log spend/fitness → weekly review.
- **Today** is the home: due/in-progress, money pulse (when exists), quick capture.
- One primary action per view; secondary actions quieter.
- **States are design:** loading, empty (with next action), error (cause + retry), success when it matters.
- Copy: short, specific errors (“Couldn’t save expense — try again”), not “Error occurred.”

## Visual (Tailwind)

- Cohesive spacing scale; reuse tokens/patterns once they exist.
- Avoid generic AI UI: purple gradients, interchangeable card grids, novelty font stacks, glow spam, emoji decoration.
- Brand/product presence on primary surfaces without dashboard clutter in the first viewport of marketing-like pages.
- Responsive: mobile-first; usable touch targets.

## Accessibility

- Real controls (`button`, `a`, labels) — not `div` + onClick.
- Keyboard focus visible; Escape closes dismissible layers.
- Do not convey meaning by color alone.

## Implementation boundary

- UI calls the API via the shared client; no business rules in components.
- Prefer composing small components over mega-pages.

## Exit criteria

- [ ] Happy path clear in <10 seconds for the target job
- [ ] Empty/loading/error covered
- [ ] Keyboard + labels checked for interactive elements
