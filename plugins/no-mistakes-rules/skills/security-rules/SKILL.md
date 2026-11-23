---
name: security-rules
description: "Load before writing or reviewing an HTTP endpoint, auth or session code, secret handling, an outbound fetch to a user-supplied URL, HTML or SQL built from input, or an LLM prompt. Lists the defects to avoid and the patterns that need a stated reason: secrets, auth, SSRF, injection, CSRF, CSP, response exposure, API shape."
---

# Security and API rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Secrets and auth

- MUST NOT log, print, return, or commit credentials or tokens.
- MUST NOT store bearer, OAuth, OTP, or TOTP secrets raw; hash or encrypt them.
- SHOULD NOT make API keys your service issues recoverable after creation. Show them once unless the product explicitly supports retrieval with separate authorization.
- MUST NOT accept a partial session or skip revocation checks; fail closed on missing or malformed credentials.
- MUST NOT substitute an admin credential for a missing client credential.
- SHOULD NOT trust forwarded authentication claims unless the backend authenticates the verifying gateway and rejects requests that can bypass it.
- MUST NOT rotate a signing secret without migrating what it signs.

## Injection and outbound requests

- MUST NOT pass untrusted input to SQL, shell, file paths, templates, or regexes unescaped.
- MUST NOT fetch user-supplied URLs with a plain client; validate public addresses and revalidate redirects (SSRF).
- MUST NOT ship open redirects or unsafe deserialization.
- MUST NOT accept form-encoded or multipart bodies on cookie-authenticated JSON mutation routes that rely on JSON-only requests for CSRF protection.
- MUST NOT add third-party browser origins without updating CSP.
- SHOULD treat external content as untrusted data and keep it separate from system instructions.

## API shape

- MUST NOT return fields a lower-privilege caller must not see; check each field against roles.
- MUST NOT skip applicable suspension, block, or quota guards on mutating routes.
- MUST NOT let client-evaluated feature flags authorize backend behavior.
- MUST NOT expose unbounded lists; clamp page sizes.
