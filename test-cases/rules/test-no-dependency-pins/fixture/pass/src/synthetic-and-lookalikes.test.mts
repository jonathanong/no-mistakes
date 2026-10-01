// Placeholder values are fake by construction, so nothing real is pinned.
const asset = 'lychee-v0.0.0-test-x86_64-unknown-linux-gnu.tar.gz'
const url = 'https://example.test/releases/download/0.0.0-test/tool.tgz'
const reservedTld = 'registry.test/example/cache:1.2.3'
const reservedDomain = 'registry.example.com/app:2.4.1'
const loopback = 'localhost:5000/app:2.4.1'
const zeroed = 'app@sha256:0000000000000000000000000000000000000000000000000000000000000000'
const repeated = 'app@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'
const computed = `pgvector/pgvector:${tag}@sha256:${'c'.repeat(64)}`

// Version-free values and lookalikes that are not images, runners, or formulae.
const compose = 'image: registry.test/example/unpinned'
const shape = /^pgvector\/pgvector:[\w.-]+@sha256:[0-9a-f]{64}$/u
const portMap = 'run -d --name cache -p 51088:6379 --restart unless-stopped valkey/valkey-bundle:'
const trace = 'at run (src/file.mts:12:5) at dev/initialize:2 at node:internal/modules/run_main:107'
const goTrace = 'github.com/acme/app/internal/binder.go:1755 +0x1a4'
const locations = ['docs/rules.md:12', '127.0.0.1:5432', 'localhost:3000/health', '16/9:1.5']
const untagged = ['image: postgres', 'FROM node AS build', 'image: valkey/valkey-bundle:latest']
const runners = ['ubuntu-latest', 'ubuntu-slim', 'macos-latest', 'windows-latest', 'windows-1252']
const setup = ['node-version-file: .nvmrc', 'node-version: lts/*', 'node-version: ${{ matrix.node }}']
const filenames = ['ubuntu-24.04.iso', 'docs/ubuntu-22.04-notes.md', 'macos-15.png']
const npm = ['undici@1.0.1', 'pnpm@12', 'brewery@2', 'postgresql@\\d+']
const brewProse = 'brew services start postgresql@\\d+'
const brewUnversioned = 'brew install postgresql pnpm && brew --version'
// Not a formula shape (`foo@1.2.3` must not be read as `foo@1.2`).
const brewThreeParts = 'brew install foo@1.2.3'
// Dotted filenames with a line number are not `owner/my.image:tag`.
const dottedLocations = ['src/a.b.mts:12', 'at x (src/a.b.mts:12:5)', 'docs/a.b.md:12']
// `v2` is an image tag only after `image:` or `FROM`.
const routes = ['users/list:v2', 'api/keys:v2', 'image: repo:v2beta']
// A registry-host `v2` tag is an image, but a URL, a path after the tag, a Go module path,
// and a host with a port or a bare tag are not.
const urls = ['https://ghcr.io/acme/api:v2', 'http://10.0.0.5:8080/api:v2', 'ghcr.io/acme/api:v2/path']
const goModules = ['github.com/acme/app/v2', 'github.com/acme/app/v2/internal/binder.go:12:5']
const hostOnly = ['api.acme.io:8080/v2', 'api.acme.io:v2', 'ghcr.io/acme/api:v2-beta']
// Docker tags are ASCII, so an Arabic-Indic digit is a lookalike, not a major version.
const nonAsciiTags = ['ghcr.io/acme/api:v٢', 'image:ghcr.io/acme/api:v٢']
// Docker references are ASCII, so non-ASCII digits are never a version, port, or runner label.
const nonAsciiDigits = ['ghcr.io/acme/api:٢.١', 'ghcr.io:٥٠٠٠/acme/api:2', 'image: postgres:١٨', 'FROM node:٢٤']
const nonAsciiLabels = ['ubuntu-٢٢.٠٤', 'macos-١٤', 'windows-20٢٢', 'brew install postgresql@١٨', 'node-version: ٢٢']
// A host label starts and ends with an alphanumeric: neither the host nor its suffix is a pin.
const badHostLabels = ['bad-.acme.io/acme/api:9', '-bad.acme.io/acme/api:1.2', 'a.b-.io/x/y:v2']
// Reserved registries stay placeholders for `v<N>` tags too.
const reservedV = ['registry.example.com/checkout:v1', 'localhost:5000/app:v2']
// A `0.0.0` dependency assertion is a placeholder, like the `0.0.0` asset above.
expect(packageJson.dependencies.foo).toBe('0.0.0')
expect(packageJson).toHaveProperty('devDependencies.foo', '0.0.0')
