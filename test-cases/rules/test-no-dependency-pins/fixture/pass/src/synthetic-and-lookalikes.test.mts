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
