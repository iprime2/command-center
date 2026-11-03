---
name: feature-recon
description: >-
  Map one user flow from another application into a Command Center replication
  brief: screens, states, entities, and a feature matrix. Clean-room only.
  Use when the user wants to copy, clone, replicate, or match a feature or
  flow from another app, product, or screenshot.
---

# Feature recon

Study what another product does. Write a brief this repo can implement. Do not start coding in this skill.

## Rules

- One app, one loop. "The whole product" is not a slice. Ask which flow if it was not named.
- Public pages, docs, and the user's own account only. Never ask for a password, log into someone else's account, pass a paywall, or scrape in a loop.
- Do not read or save the other app's source, bundles, decompiled binaries, or private API traffic. Public API docs are allowed as evidence of behavior.
- Record behavior and states. Do not copy their code, logo, marketing copy, colors as a brand, or content catalogue.
- FitSage stays a separate project. Do not recon it into this repo unless the user explicitly says to merge FitSage.
- If their terms forbid using an account to build a competitor, say so and use public sources only.
- Screenshots are reference. Do not commit them. Do not put secrets or personal data in the brief.

## Write

Create `docs/replications/<slug>/` with:

- `recon.md` — sources (URL on every row), screens (`S01`…), flows (`F01`…), repeated UI parts, inferred entities with evidence and confidence, what is out of scope and why, size (S/M/L)
- `features.csv` — `feature,area,priority,original,clone,notes`. Priority is `must`, `should`, or `could`. `clone` starts as `no`. `skip` rows name things that cannot be rebuilt (their network, licensed content, hardware).

Each entity field needs evidence (which screen or public doc) or the word `guess`.

## Exit

Five lines in chat: the loop, screen and flow counts, the three hardest parts, what is skipped, and the next step (`feature-adapt`). Stop until the user confirms the brief. Do not invent a stack.
