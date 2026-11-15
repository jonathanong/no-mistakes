"""Trigger variants of the no-mistakes-rules plugin, built from plugins/ at run time.

`python3 evals/generate.py --rules-variant <name>` builds the variant plugin under
evals-variants/_plugins/<name>/ and emits the rules and off-topic cases under
evals-variants/rules-<name>/, pointed at it. Rule bodies are copied unchanged;
only descriptions or skill layout differ, so a score change is a triggering
change. See README "Rules-plugin cases".

- control: an unchanged copy, so a same-day run shares paths and cost with the
  variants.
- describe: every description opens with "Load before ..." and names concrete
  files, libraries, and activities instead of "MUST NOT / SHOULD NOT rules for".
- entry: one `coding-rules` skill whose body indexes the topics; each topic's
  rules move to references/<topic>-rules.md.
"""
import pathlib
import shutil

DESCRIBE = {
    "dependency-rules": "Load before adding, upgrading, or removing a dependency, or editing package.json, a lockfile (pnpm-lock.yaml, package-lock.json), a workspace, a Dependabot config, or an npm publish step. Lists the defects to avoid and the patterns that need a stated reason.",
    "git-rules": "Load before committing, staging, rebasing, pushing, writing a PR description, editing docs or AGENTS.md, or stating a claim about production behavior. Lists the defects to avoid and the patterns that need a stated reason: staging, history rewrites, hooks, scope, evidence for claims.",
    "github-actions-rules": "Load before writing, reviewing, or debugging anything under .github/workflows (ci.yml, release.yml, publish.yml), a composite action, a required check, or a CI failure. Lists the defects to avoid and the patterns that need a stated reason: pinning, permissions, untrusted input, timeouts, polling and gating on other workflows, required checks, concurrency, runners, CI log diagnosis.",
    "infra-rules": "Load before writing or reviewing infrastructure and ops code: OpenTofu or Terraform, IAM policies, secrets stores, provisioning or cleanup scripts, shell scripts, health probes, or code whose correctness depends on how many instances run. Lists the defects to avoid and the patterns that need a stated reason.",
    "integrations-rules": "Load before writing or reviewing code that calls, parses, or receives a third-party API or webhook (Stripe, Twilio, Slack, a partner REST API). Lists the defects to avoid and the patterns that need a stated reason: response classification, retries and idempotency, delivery records, payload mapping, timestamps and units, batch failures, signature verification.",
    "native-app-rules": "Load before writing or reviewing Swift or SwiftUI (iOS, macOS) or C# and .NET MAUI app code. Lists the defects to avoid and the patterns that need a stated reason: core-library boundaries, concurrency, localization, secure storage, release config.",
    "observability-rules": "Load before adding or reviewing a log line, a metric, a trace attribute, an alert, or a monitor (pino, winston, OpenTelemetry, Datadog, Sentry). Lists the defects to avoid and the patterns that need a stated reason: static messages and structured fields, personal data, metric cardinality, fields monitors depend on, absence alerts, context binding.",
    "playwright-rules": "Load before writing, reviewing, or de-flaking a Playwright or other browser end-to-end test, or adding data-testid attributes to UI. Lists the defects to avoid and the patterns that need a stated reason: waits, locators, test ids, setup, network mocking, timeouts.",
    "postgresql-rules": "Load before writing or reviewing anything that sends SQL to PostgreSQL: a query through pg/node-postgres, Kysely, Drizzle, Prisma, or psql; a repository or data-access function; a migration; an index; or a query plan. Lists the defects to avoid and the patterns that need a stated reason: queries, locks, transactions, schema, migrations, partitioning.",
    "queue-rules": "Load before writing or reviewing background work: a queue producer or worker (BullMQ, pg-boss, SQS), an enqueue call, a cron or nightly job, a sweep, reconciler, cleanup, backfill, or any loop that reads rows on a schedule. Lists the defects to avoid and the patterns that need a stated reason: producers, workers, leases, dedup, bounded iteration.",
    "react-rules": "Load before writing or reviewing React components, hooks, or a Next.js page, layout, route handler, or server action. Lists the defects to avoid and the patterns that need a stated reason: rendering, server and client boundaries, data fetching, caching, metadata, scripts, routing, accessibility.",
    "rollout-rules": "Load before adding a feature flag, changing an API, event, or message contract, deploying a change that spans services, or touching payments, refunds, or other irreversible actions. Lists the defects to avoid and the patterns that need a stated reason: deploy order, compatibility, string-referenced config.",
    "rust-rules": "Load before writing or reviewing Rust code, Cargo.toml, clippy settings, or benchmarks, or before running heavy cargo builds. Lists the defects to avoid and the patterns that need a stated reason: lints, test layout, file size, parallelism, hot loops, caching, build hygiene, performance claims.",
    "security-rules": "Load before writing or reviewing an HTTP endpoint, auth or session code, secret handling, an outbound fetch to a user-supplied URL, HTML or SQL built from input, or an LLM prompt. Lists the defects to avoid and the patterns that need a stated reason: secrets, auth, SSRF, injection, CSRF, CSP, response exposure, API shape.",
    "testing-rules": "Load before writing, reviewing, or fixing any automated test or test helper (Vitest, Jest, node:test, backend integration tests, React Testing Library, XCTest, xUnit). Lists the test defects to avoid and the patterns that need a stated reason: assertions, mocks, fixtures, isolation, timers, snapshots, coverage. Browser tests also load playwright-rules.",
    "typescript-rules": "Load before writing or reviewing TypeScript, JavaScript, or Node.js code: async functions and promises, error handling, HTTP clients (fetch, undici, axios), Redis or Valkey caches, process lifecycle, package boundaries. Lists the defects to avoid and the patterns that need a stated reason.",
}

ENTRY_DESCRIPTION = "Load before writing, reviewing, or fixing code, tests, SQL, migrations, CI workflows, background jobs, infrastructure, or dependencies. Indexes the defects to avoid by topic: tests, PostgreSQL, queues and scheduled jobs, GitHub Actions, TypeScript and Node, React and Next.js, security, third-party APIs, rollout, observability, infra, dependencies, git, Rust, native apps, and Playwright."

ENTRY_SKILL = "coding-rules"
VARIANTS = ("control", "describe", "entry")


def _split(text: str) -> tuple[str, str]:
    _, front, body = text.split("---\n", 2)
    return front, body


def _description(front: str) -> str:
    for line in front.splitlines():
        if line.startswith("description: "):
            return line.removeprefix("description: ").strip('"')
    raise ValueError("no description")


def build(name: str, src: pathlib.Path, out: pathlib.Path) -> str | None:
    """Write variant `name` of the plugin at `src` to `out`. Returns the skill
    every should-fire case expects, or None when each case expects its topic."""
    if name not in VARIANTS:
        raise SystemExit(f"unknown rules variant {name!r}; one of {', '.join(VARIANTS)}")
    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(src, out)
    skills = out / "skills"
    if name == "describe":
        for skill, desc in DESCRIBE.items():
            path = skills / skill / "SKILL.md"
            front, body = _split(path.read_text())
            old = _description(front)
            path.write_text("---\n" + front.replace(old, desc, 1) + "---\n" + body)
    if name != "entry":
        return None
    rows, refs = [], {}
    for path in sorted(skills.glob("*/SKILL.md")):
        front, body = _split(path.read_text())
        topic = path.parent.name
        refs[topic] = body.lstrip("\n")
        rows.append(f"| `references/{topic}.md` | {_description(front)} |")
    shutil.rmtree(skills)
    entry = skills / ENTRY_SKILL
    (entry / "references").mkdir(parents=True)
    for topic, body in refs.items():
        (entry / "references" / f"{topic}.md").write_text(body)
    (entry / "SKILL.md").write_text(
        f'---\nname: {ENTRY_SKILL}\ndescription: "{ENTRY_DESCRIPTION}"\n---\n\n'
        "# Coding rules\n\n"
        "Before you write or review, read the reference for every topic the task "
        "touches. MUST NOT is a defect. SHOULD NOT needs a stated, measured reason.\n\n"
        "| Reference | Covers |\n| --- | --- |\n" + "\n".join(rows) + "\n"
    )
    return ENTRY_SKILL
