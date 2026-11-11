---
name: security-rules
description: "MUST NOT / SHOULD NOT rules for application security and HTTP APIs: secrets, auth and sessions, SSRF, injection, CSRF, CSP, LLM prompts, response exposure, and API shape. Use when writing or reviewing an endpoint, auth code, secret handling, outbound fetches, or LLM calls."
---

# Security and API rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Secrets and auth

- MUST NOT log, print, return, or commit credentials or tokens.
- MUST NOT store bearer, OAuth, OTP, or TOTP secrets raw; hash or encrypt through one shared module.
- MUST NOT make API keys recoverable after creation; show them once.
- MUST NOT accept a partial session or skip revocation checks; fail closed on missing or malformed credentials.
- MUST NOT substitute an admin credential for a missing client credential.
- MUST NOT let edge-verified claims grant access; the backend decides.
- MUST NOT rotate a signing secret without migrating what it signs.

## Injection and outbound requests

- MUST NOT pass untrusted input to SQL, shell, file paths, templates, or regexes unescaped.
- MUST NOT fetch user-supplied URLs with a plain client; validate public addresses and revalidate redirects (SSRF).
- MUST NOT ship open redirects or unsafe deserialization.
- MUST NOT accept form-encoded or multipart bodies on mutating JSON routes (CSRF).
- MUST NOT add third-party browser origins without updating CSP.
- MUST NOT feed external content to an LLM without sanitizing it, or put user data in system prompts.

## API shape

- MUST NOT return fields a lower-privilege caller must not see; check each field against roles.
- MUST NOT skip suspension, block, or quota guards on mutating routes.
- MUST NOT gate backend behavior on frontend feature flags.
- MUST NOT expose unbounded lists; paginate with opaque cursors and clamp limits.
- MUST NOT let a schema drift from its handler, or widen it to hide drift.
- MUST NOT expose internal ids as the user-facing identifier when a stable slug exists.
- SHOULD NOT put business logic in route handlers, or response and cookie handling in services.
- SHOULD NOT use RPC-style bodies (`{ action: "verb" }`); use resource paths.
- SHOULD NOT hand-build cache headers per route; centralize caching.
