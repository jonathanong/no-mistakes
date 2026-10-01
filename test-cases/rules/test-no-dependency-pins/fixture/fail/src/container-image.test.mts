expect(compose).toContain('image: valkey/valkey-bundle:9.1.0')
expect(compose).toContain("image: 'otel/opentelemetry-collector-contrib:0.153.0'")
expect(compose).toContain('image: valkey/valkey-bundle:9')
expect(dockerfile).toContain('FROM postgres:18-alpine')
expect(dockerfile).toContain('FROM --platform=linux/amd64 node:24.11.1-bookworm-slim AS build')
expect(image).toBe('ghcr.io/acme/api:v2.4.1')
expect(image).toBe('registry.internal:5000/acme/api:2.4.1')
expect(image).toBe('pgvector/pgvector:pg18')
expect(image).toBe('ghcr.io/acme/api:5000')
// The tag is a pin even when the digest after it is interpolated.
expect(log).toContain('mirror.gcr.io/library/node:26-trixie-slim@sha256:${imageDigest}')
expect(image).toBe('docker.io/library/redis@sha256:06ada57c26aa5cf429e9f2c0a99e3e4a42daecd45fc4c955d7c1399ab4227ae8')
expect(image).toBe('redis@sha256:e0ef56d6eb603d9ffe87e74c28b8210b0ea93839e2ea4fb64f1fc2c66e95e391')
expect(images).toEqual(['valkey/valkey-bundle:9.1.0', 'otel/collector:0.153.0'])
// Docker path components may contain dots, so `my.image` is one repository name.
expect(compose).toContain('image: owner/my.image:1.2.3')
// A major-only `v2` tag is an image after `image:`, where `users/list:v2` cannot be a key.
expect(compose).toContain('image: repo:v2')
expect(compose).toContain('image: owner/repo:v2')
// Under a registry host a major-only `v2` tag is an image anywhere on the line: it cannot be
// an API key there. The slash-only `owner/repo:v2` beside it is not a pin, only the host path is.
expect(image).toBe('ghcr.io/acme/api:v2')
expect(image).toBe('registry.internal:5000/acme/api:v3')
expect(images).toEqual(['owner/repo:v2', 'ghcr.io/acme/api:v4'])
// With no space or quote after `image:`, a tight-context pattern still reads the host tag.
expect(compose).toContain('image:ghcr.io/acme/api:v5')
// A registry host label may hold hyphens inside it, and an IP address is a host.
expect(image).toBe('registry-eu.acme.io/acme/api:2.4.1')
expect(image).toBe('10.0.0.5:5000/acme/api:2')
// A non-ASCII character ends a reference and is a boundary on either side, so the ASCII part is still a pin.
expect(image).toBe('ghcr.io/acme/api:v6β')
expect(image).toBe('βghcr.io/acme/api:7')
expect(compose).toContain('éimage: postgres:18')
