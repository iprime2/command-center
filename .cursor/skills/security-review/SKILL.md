---
name: security-review
description: >-
  Security review for Command Center: secrets, authn/authz, injection, XSS,
  error leakage, expense/money paths. Use for auth, expenses, user input, or
  before shipping sensitive changes.
---

# Security Review

## Checklist

- [ ] No hardcoded secrets; env-based config with fail-fast if missing
- [ ] All external inputs validated
- [ ] SQL parameterized (SQLx); no string-concat queries
- [ ] XSS: no unsanitized HTML from users
- [ ] Authn required where expected; **authz** checks resource ownership
- [ ] Sensitive errors not returned to clients
- [ ] Expense/money: amount/category/date validated; user-scoped queries
- [ ] CORS/cookies/CSRF posture sensible for the cookie/token model in use
- [ ] Dependencies: no known reckless trust of unverified user uploads (later)

## If issue found

1. Stop feature expansion.
2. Label severity: **Critical** / **High** / **Medium**.
3. Fix Critical/High before merge/sign-off.
4. Rotate any exposed secrets.

## Output

List findings with severity, location, and required fix. Explicit **no findings** if clean.
