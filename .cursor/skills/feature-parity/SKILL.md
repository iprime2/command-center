---
name: feature-parity
description: >-
  Score a Command Center slice against its feature matrix: must-have behavior
  present or missing, with test and browser evidence. Use after implementing a
  replicated flow, or when the user asks if the clone or port matches the
  other app.
---

# Feature parity

Compare this repo's behavior to `docs/replications/<slug>/features.csv`. Do not compare pixels, branding, or copy to the other product.

## Update the matrix

Set `clone` to `yes`, `partial`, or `no` for each in-scope row. `partial` counts as half. Leave `skip` rows out of the score. Do not mark `yes` without evidence.

## Evidence

For every `must` row claimed `yes` or `partial`:

- the API test, command, and result, or
- the browser steps and what happened

Missing `must` rows mean the slice is not done. Say which ones.

## Exit

Post the counts (must done, must partial, must missing, should still open) and the next gap. Hand a missing must-have back as a normal slice (`tdd-workflow`). Do not open a new stack, a deploy, or a rename of the product from this skill.
