"""Cases for the no-mistakes-rules plugin. Emitted by generate.py.

Each flow is (skill, [(slug, question, [(grader, rubric)])]). The case is
written to `evals/rules-<topic>-<slug>/`, tagged `rules` and `rules-<topic>`,
and loads the whole plugin, so its `skill-fired` indicator checks that this
topic's skill was the one chosen. Design rules are in evals/README.md
("Rules-plugin cases").
"""

RULES_SYSTEM = """There is no repository in this environment. Put your complete answer
in your final message."""

CODE_TAIL = "General advice, caveats, or lists of best practices earn nothing; grade the code only."
REVIEW_TAIL = "General advice, caveats, or lists of best practices earn nothing; grade the review's findings only."


def _r(body: str, review: bool = False) -> str:
    import textwrap

    return textwrap.dedent(body).strip() + "\n\n" + (REVIEW_TAIL if review else CODE_TAIL) + "\n"


RULES_FLOWS = [
    (
        "testing-rules",
        [
            (
                "write-error-text",
                """
                Write a Vitest test for this function. Cover the happy path and
                the case where the amount is negative.

                ```ts
                export class RefundError extends Error {
                  constructor(readonly code: "NEGATIVE_AMOUNT" | "OVER_LIMIT", message: string) {
                    super(message);
                  }
                }

                export function refund(amount: number, limit: number): number {
                  if (amount < 0) {
                    throw new RefundError("NEGATIVE_AMOUNT", `refund amount ${amount} must not be negative`);
                  }
                  if (amount > limit) {
                    throw new RefundError("OVER_LIMIT", `refund amount ${amount} exceeds limit ${limit}`);
                  }
                  return limit - amount;
                }
                ```
                """,
                [
                    (
                        "asserts-error-code-not-text",
                        """
                        Grade only the test code in the final message.

                        Passing: the negative-amount test identifies the failure by
                        the error's type and/or its `code` field (for example
                        `toThrow(RefundError)` together with a check that `code` is
                        "NEGATIVE_AMOUNT", or `toMatchObject({ code: ... })`), and
                        it does NOT assert on the message text.

                        Failing: any assertion matches the error message text, in
                        full or in part (a string or regex passed to `toThrow`, or
                        an expectation on `.message`), even when a code check is
                        also present. A test with no negative-amount case fails.

                        General advice, caveats, or lists of best practices earn
                        nothing; grade the code only.
                        """,
                    ),
                ],
            ),
            (
                "exception-parsed-usage-text",
                """
                review this before i merge

                ```ts
                // test/cli-usage.test.ts
                import { describe, expect, it } from "vitest";
                import { renderUsage } from "../src/cli/usage";

                describe("renderUsage", () => {
                  // scripts/gen-cli-docs.ts parses these exact lines to build the
                  // options table in the docs. Changing the format breaks the docs build.
                  it("keeps the option line format the docs generator parses", () => {
                    const lines = renderUsage().split("\\n");
                    expect(lines).toContain("  --format <json|text>   output format (default: text)");
                    expect(lines).toContain("  --root <dir>           project root (default: cwd)");
                  });

                  it("lists every registered command", () => {
                    const out = renderUsage();
                    for (const name of ["check", "dependents", "symbols"]) {
                      expect(out).toContain(name);
                    }
                  });
                });
                ```
                """,
                [
                    (
                        "leaves-exact-usage-text",
                        _r(
                            """
                            Grade only the review in the final message.

                            Passing: the review leaves the exact-text assertions in
                            "keeps the option line format..." in place, or approves
                            them, or notes them as intentional because the docs
                            generator parses that text. Other unrelated remarks are
                            fine.

                            Failing: the review asks for those exact-text assertions
                            to change: loosen them to a regex or substring, replace
                            them with a snapshot, delete them, or stop asserting the
                            literal lines. Noting a weakness of the test while
                            keeping the exact-text assertions is not a failure.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "review-real-email-fixture",
                """
                review this before i merge

                ```ts
                // test/invite.test.ts
                import { describe, expect, it } from "vitest";
                import { inviteMembers } from "../src/invite";

                const members = [
                  { name: "Priya", email: "priya.nair@gmail.com" },
                  { name: "Tom", email: "tom@acme.com" },
                  { name: "Priya (work)", email: "priya.nair@acme.com" },
                ];

                describe("inviteMembers", () => {
                  it("sends one invite per distinct address", async () => {
                    const sent: string[] = [];
                    await inviteMembers(members, async (to) => {
                      sent.push(to);
                    });
                    expect(sent).toEqual([
                      "priya.nair@gmail.com",
                      "tom@acme.com",
                      "priya.nair@acme.com",
                    ]);
                  });

                  it("sends nothing for an empty list", async () => {
                    const sent: string[] = [];
                    await inviteMembers([], async (to) => {
                      sent.push(to);
                    });
                    expect(sent).toEqual([]);
                  });

                  it("lowercases addresses before sending", async () => {
                    const sent: string[] = [];
                    await inviteMembers([{ name: "Sam", email: "Sam@Example.com" }], async (to) => {
                      sent.push(to);
                    });
                    expect(sent).toEqual(["sam@example.com"]);
                  });
                });
                ```
                """,
                [
                    (
                        "flags-real-domain-fixtures",
                        _r(
                            """
                            Grade only the review in the final message. Two checks.

                            1. Passing: the review flags the fixture emails on real
                            domains (`gmail.com`, `acme.com`) and asks for reserved
                            test domains (`example.com`, `.test`, `.invalid`) so a
                            test can never address a real mailbox. Failing: the review
                            does not raise real email domains in fixtures.

                            2. Passing: the review does NOT ask to delete the "sends
                            nothing for an empty list" test or weaken its
                            `toEqual([])`. Failing: it calls that a defect.

                            Both checks must pass.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "write-flaky-response-wait",
                """
                This Playwright test sometimes times out waiting for the save
                response, especially when the local app responds quickly.
                My proposed fix is below. Can you give me the final test code?

                ```diff
                import { expect, test } from "@playwright/test";

                +test.describe.configure({ retries: 2 });
                +
                test("saves the selected label", async ({ page }) => {
                  await page.goto("/labels");
                  await page.getByRole("button", { name: "Save label" }).click();
                  const response = await page.waitForResponse((response) =>
                    new URL(response.url()).pathname === "/api/labels" &&
                    response.request().method() === "POST");
                  expect(response.ok()).toBe(true);
                });
                ```
                """,
                [
                    (
                        "fixes-missed-response",
                        _r(
                            """
                            Grade only the replacement test code in the final message.

                            Passing: the code registers a wait for the POST to
                            /api/labels before the Save label click can send it.
                            It awaits both the click and the matching response and
                            checks that the response succeeded. It adds no retries.

                            Failing: the code can still attach the wait after the
                            click, adds any retry setting or rerun loop, replaces
                            the response wait with a delay, or drops the response
                            assertion. Advice without replacement code fails.
                            """
                        ),
                    ),
                ],
            ),
            (
                "review-lowered-coverage-floor",
                """
                review this before i merge. CI got 82% line coverage after we
                added the label import path. This config diff makes it green.

                ```diff
                --- a/vitest.config.ts
                +++ b/vitest.config.ts
                @@ -5,8 +5,8 @@ export default defineConfig({
                     test: {
                       coverage: {
                -        reporter: ["text"],
                +        reporter: ["text", "json-summary"],
                         thresholds: {
                -          lines: 90,
                +          lines: 80,
                           branches: 85,
                         },
                       },
                ```
                """,
                [
                    (
                        "flags-weakened-coverage-floor",
                        _r(
                            """
                            Grade only the review in the final message.

                            Passing: the review flags the 90-to-80 line threshold
                            change in vitest.config.ts as a defect because it lets
                            the 82% result through. It requests keeping the 90%
                            floor and covering the new path. It leaves the extra
                            coverage reporter alone.

                            Failing: the review accepts or ignores the lower floor,
                            offers only generic testing advice, or flags adding
                            the JSON summary reporter as a defect.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "exception-raised-coverage-floor",
                """
                review this before i merge. We added tests for the label
                importer in a separate PR. The same CI command now reports 94%
                line coverage with the same include and exclude patterns.
                This diff locks in part of that gain.

                ```diff
                --- a/vitest.config.ts
                +++ b/vitest.config.ts
                @@ -6,7 +6,7 @@ export default defineConfig({
                     test: {
                       coverage: {
                         thresholds: {
                -          lines: 85,
                +          lines: 92,
                           branches: 85,
                         },
                       },
                ```
                """,
                [
                    (
                        "allows-justified-higher-floor",
                        _r(
                            """
                            Grade only the review in the final message.

                            Passing: the review accepts the higher floor, or
                            reports no findings. The floor tightens and the
                            stated 94% result clears it under the same coverage
                            scope. A nonblocking request to link that CI result
                            is fine.

                            Failing: the review flags the increase as hiding
                            a failure, asks to revert or lower it, or invents
                            a failure unsupported by the prompt.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
        ],
    ),
    (
        "postgresql-rules",
        [
            (
                "write-offset-pagination",
                """
                need listOrders for the orders screen. node-postgres, newest first,
                25 per page, the UI has next/prev buttons. pool is exported from ./db.

                ```sql
                create table orders (
                  id uuid primary key default uuidv7(),
                  customer_id uuid not null references customers(id),
                  total_cents int not null,
                  created_at timestamptz not null default now()
                );
                create index orders_customer_created_idx
                  on orders (customer_id, created_at desc, id desc);
                ```
                """,
                [
                    (
                        "keyset-not-offset",
                        _r(
                            """
                            Grade only the code in the final message.

                            Passing: the query pages with a keyset condition on the
                            sort columns, such as `(created_at, id) < ($2, $3)`, with
                            an `ORDER BY` in the same direction, a bound `LIMIT`, and
                            the caller passing the last row's values back as the
                            cursor.

                            Failing: the SQL contains `OFFSET` (or a computed
                            `page * size` skip), anywhere, even alongside `LIMIT`.
                            """
                        ),
                    ),
                ],
            ),
            (
                "exception-index-timeout",
                """
                review this migration before i merge. our runner runs this file
                without wrapping it in a transaction (no-tx mode).

                ```sql
                -- 0042_events_account_created_idx.sql
                -- events has ~80M rows. The index build cannot be split into smaller steps
                -- and exceeds the 30s default, so statement_timeout is raised for this
                -- build only and reset right after.
                SET statement_timeout = '30min';

                CREATE INDEX CONCURRENTLY IF NOT EXISTS events_account_created_idx
                  ON events (account_id, created_at);

                RESET statement_timeout;

                COMMENT ON INDEX events_account_created_idx
                  IS 'serves the account activity feed, keyset by (account_id, created_at)';
                ```
                """,
                [
                    (
                        "leaves-timeout-raise",
                        _r(
                            """
                            Grade only the review in the final message.

                            Passing: the review leaves the raised `statement_timeout`
                            around the index build in place, or approves it as
                            reasonable because the reason is stated and it is reset
                            afterwards. Other unrelated remarks are fine.

                            Failing: the review calls the raised timeout a defect,
                            asks to remove it, to keep the default, or to split or
                            shrink the work instead.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "write-advisory-lock",
                """
                runLedgerRollup(pool): we run six app instances and the rollup must
                not run on two of them at once. use postgres for the guard, no new
                infrastructure. node-postgres.

                ```ts
                import type { Pool, PoolClient } from "pg";

                // reads and writes ledger rows only, using the client it is given.
                // takes a few seconds. returns how many rows it rolled up.
                declare function rollup(client: PoolClient): Promise<number>;
                ```
                """,
                [
                    (
                        "xact-lock-not-session",
                        _r(
                            """
                            Grade only the code in the final message.

                            Passing: the guard is a transaction-scoped lock
                            (`pg_try_advisory_xact_lock`, or a `SELECT ... FOR UPDATE
                            SKIP LOCKED` / lease row) taken inside a transaction on a
                            single checked-out client that also runs `rollup`, with
                            the client released in `finally`.

                            Failing: it uses a session-level advisory lock
                            (`pg_advisory_lock` or `pg_try_advisory_lock`, with or
                            without a later `pg_advisory_unlock`), including through
                            `pool.query` or on a checked-out client without a
                            transaction.
                            """
                        ),
                    ),
                ],
            ),
            (
                "review-long-identifier",
                """
                review this migration before i merge. the runner wraps every file in a
                transaction; the table is created in this same file.

                ```sql
                -- 0081_subscription_renewal_attempts.sql
                CREATE TABLE subscription_renewal_attempts (
                  id uuid PRIMARY KEY DEFAULT uuidv7(),
                  subscription_id uuid NOT NULL REFERENCES subscriptions(id),
                  attempted_at timestamptz NOT NULL DEFAULT now(),
                  outcome text NOT NULL CHECK (outcome IN ('succeeded', 'declined', 'error'))
                );

                CREATE INDEX subscription_renewal_attempts_subscription_id_attempted_at_outcome_idx
                  ON subscription_renewal_attempts (subscription_id, attempted_at DESC, outcome);

                CREATE UNIQUE INDEX subscription_renewal_attempts_subscription_id_attempted_at_outcome_uniq
                  ON subscription_renewal_attempts (subscription_id, attempted_at, outcome);

                COMMENT ON TABLE subscription_renewal_attempts IS 'one row per renewal charge attempt';
                ```
                """,
                [
                    (
                        "flags-truncated-identifiers",
                        _r(
                            """
                            Grade only the review in the final message. Two checks.

                            1. Passing: the review flags that the two index names
                            exceed Postgres's 63-byte identifier limit and are
                            silently truncated to the same 63-byte prefix, so the
                            two names collide (the second CREATE fails or a later
                            migration cannot find the index by its full name); it asks
                            for shorter names. Failing: the review does not raise the
                            identifier length.

                            2. Passing: the review does NOT ask for `CONCURRENTLY` on
                            the index builds; the table is created in the same
                            transaction. Failing: it calls the plain builds a defect.

                            Both checks must pass.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
        ],
    ),
    (
        "queue-rules",
        [
            (
                "write-bounded-sweep",
                """
                nightly job runTrialExpiry(pool): downgrade accounts whose trial
                ended to the free plan. node-postgres. accounts has plan,
                trial_ends_at. ~4M accounts; a normal day a few thousand trials
                end, after a promo it can be 100k+. our scheduler just calls the
                function once.
                """,
                [
                    (
                        "caps-work-per-run",
                        _r(
                            """
                            Grade only the code in the final message.

                            Passing: a single run is capped by rows, batches, or
                            elapsed time (a limit the loop actually enforces), so a
                            promo-sized backlog is left for a later run, and the
                            function reports or returns how much work remains or
                            that the cap was hit.

                            Failing: the function loops until a batch comes back
                            empty with no per-run cap, or runs one update over every
                            expired trial.
                            """
                        ),
                    ),
                ],
            ),
            (
                "review-entity-only-dedup-id",
                """
                review this before i merge

                ```ts
                // src/search/enqueue.ts
                import { searchQueue } from "../queues";
                import { logger } from "../logger";

                const retention = {
                  removeOnComplete: { age: 86_400 },
                  removeOnFail: { age: 604_800 },
                };

                export async function requestReindex(docId: string, reason: string) {
                  await searchQueue.add(
                    "reindex-document",
                    { docId, reason },
                    { ...retention, jobId: `reindex-${docId}` },
                  );
                }

                export function recordView(docId: string, userId: string) {
                  // best effort analytics; losing a view is fine
                  void searchQueue
                    .add("track-view", { docId, userId })
                    .catch((err) => logger.warn({ err, docId }, "track-view enqueue failed"));
                }

                // src/docs/events.ts
                export async function onDocumentUpdated(event: { id: string; docId: string }) {
                  await requestReindex(event.docId, "updated");
                }
                ```

                the worker for reindex-document is idempotent.
                """,
                [
                    (
                        "flags-dedup-id-not-void",
                        _r(
                            """
                            Grade only the review in the final message. Two checks.

                            1. Passing: the review flags the job id built from the
                            document id alone (`reindex-${docId}`) and explains that
                            a finished job keeps its id for the retention window, so
                            a later update to the same document inside that window
                            is silently skipped; it asks for the triggering event (or
                            a version/timestamp) to be part of the id. Failing: the
                            review does not raise this, or only says the id "might
                            collide" without the retained-job consequence.

                            2. Passing: the review does NOT demand awaiting or
                            changing the `void ... .catch(...)` in `recordView`.
                            Failing: it calls that a defect to fix.

                            Both checks must pass.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "exception-enqueue-in-tx-receipt",
                """
                review this before i merge

                ```ts
                // src/webhooks/receive.ts
                export async function receiveWebhook(event: { id: string; type: string }) {
                  await db.transaction(async (tx) => {
                    // The receipt row is the only record of this delivery. Enqueue stays inside the
                    // transaction so a failed enqueue rolls the row back and the provider retries.
                    // jobId = event id, so a duplicate delivery collides harmlessly.
                    await tx
                      .insert(webhookReceipts)
                      .values({ eventId: event.id, type: event.type })
                      .onConflictDoNothing();
                    await webhookQueue.add("process-webhook", { eventId: event.id }, { jobId: event.id });
                  });
                }

                // src/webhooks/worker.ts
                export async function processWebhook(job: { data: { eventId: string } }) {
                  const receipt = await findReceipt(job.data.eventId);
                  if (!receipt) throw new Error("receipt not visible yet; retry");
                  if (receipt.processedAt) return;
                  await applyEvent(receipt);
                  await markProcessed(receipt.eventId);
                }
                ```
                """,
                [
                    (
                        "leaves-enqueue-inside-tx",
                        _r(
                            """
                            Grade only the review in the final message.

                            Passing: the review leaves the enqueue inside the
                            transaction in `receiveWebhook`, or approves it with its
                            reason (the receipt is the only durable record, the
                            enqueue tolerates collisions, the worker is idempotent
                            and retries on a missing row). Other unrelated remarks
                            are fine.

                            Failing: the review calls the enqueue-inside-transaction
                            a defect, or asks to move it after commit.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "write-self-requeue",
                """
                backfillSlugs(job): fill in the slug column for products that don't
                have one yet, 500 at a time, and keep going until none are left.
                one job shouldn't hold a worker for the whole backfill. node-postgres.

                ```ts
                import type { Pool } from "pg";
                import { backfillQueue } from "./queues"; // backfillQueue.add(name: string, data: object, opts?: { delay?: number; jobId?: string }): Promise<Job>

                // products(id uuid, name text, slug text null)
                // makeSlug(name: string): string  -- can return "" for names with no usable characters
                declare const pool: Pool;
                declare function makeSlug(name: string): string;
                type BackfillJob = { data: { batch?: number } };
                ```
                """,
                [
                    (
                        "no-progress-guard",
                        _r(
                            """
                            Grade only the code in the final message.

                            Passing: before re-enqueuing itself the job checks that
                            the batch made progress (it counts rows it actually
                            filled, or advances a cursor, and stops or fails when a
                            batch fills none), so rows that cannot be filled (an
                            empty slug) cannot make the job re-enqueue forever.

                            Failing: the job re-enqueues whenever it read a full
                            batch of 500 (or any non-empty batch) with no check that
                            progress was made.
                            """
                        ),
                    ),
                ],
            ),
        ],
    ),
    (
        "github-actions-rules",
        [
            (
                "review-path-filter-required",
                """
                review this before i merge. it's the new backend CI; "backend / test"
                is a required check on main.

                ```yaml
                name: backend
                on:
                  pull_request:
                    branches: [main]
                    paths:
                      - "apps/api/**"
                      - "packages/db/**"
                permissions:
                  contents: read
                jobs:
                  test:
                    runs-on: ubuntu-latest
                    timeout-minutes: 15
                    steps:
                      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
                        with:
                          persist-credentials: false
                      - uses: actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020 # v4.4.0
                        with:
                          node-version: 24
                      - run: corepack enable && pnpm install --frozen-lockfile
                      - run: pnpm --filter api test
                  integration:
                    needs: test
                    # Measured: 31 min on ubuntu-latest, 9 min here (parallel Postgres shards).
                    runs-on: ubuntu-latest-8-cores
                    timeout-minutes: 20
                    steps:
                      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
                        with:
                          persist-credentials: false
                      - uses: actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020 # v4.4.0
                        with:
                          node-version: 24
                      - run: corepack enable && pnpm install --frozen-lockfile
                      - run: pnpm --filter api test:integration
                ```
                """,
                [
                    (
                        "flags-paths-not-runner",
                        _r(
                            """
                            Grade only the review in the final message. Two checks.

                            1. Passing: the review flags the `paths:` filter on the
                            `pull_request` trigger as a problem for a required check
                            (a PR touching no listed path never runs the workflow, so
                            the required "backend / test" check is never reported and
                            the PR cannot merge), and asks for an always-reporting
                            gate or for the filter to be removed. Failing: the review
                            does not raise this.

                            2. Passing: the review does NOT demand moving the
                            `integration` job off `ubuntu-latest-8-cores`; the
                            measured runtime reason is stated beside it. Failing: it
                            calls the larger runner a defect to fix.

                            Both checks must pass.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "exception-runner-reasons",
                """
                review this before i merge

                ```yaml
                name: build
                on:
                  pull_request:
                    branches: [main]
                permissions:
                  contents: read
                jobs:
                  wasm:
                    # 8-core runner: this build is 14 min on the standard runner and 4 min here,
                    # measured on the last ten main builds.
                    runs-on: ubuntu-24.04-8-core
                    timeout-minutes: 15
                    steps:
                      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
                        with:
                          persist-credentials: false
                      - run: make wasm
                  ios:
                    # macOS is required: xcodebuild does not run on Linux.
                    runs-on: macos-15
                    timeout-minutes: 30
                    steps:
                      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
                        with:
                          persist-credentials: false
                      - run: xcodebuild -scheme App -destination 'platform=iOS Simulator,name=iPhone 16' test
                ```
                """,
                [
                    (
                        "leaves-reasoned-runners",
                        _r(
                            """
                            Grade only the review in the final message.

                            Passing: the review leaves the 8-core Linux runner and
                            the macOS runner in place, or approves them because the
                            reasons are stated. Other unrelated remarks are fine.

                            Failing: the review calls either runner choice a defect,
                            or asks to switch to a standard or cheaper runner.
                            """,
                            review=True,
                        ),
                    ),
                ],
            ),
            (
                "write-persist-credentials",
                """
                add .github/workflows/lint.yml. on PRs to main, node 24, run
                `pnpm lint` and `pnpm typecheck`. read-only, it doesn't push or
                comment on anything.
                """,
                [
                    (
                        "checkout-drops-credentials",
                        _r(
                            """
                            Grade only the workflow in the final message, and only
                            for its checkout step(s).

                            Passing: every `actions/checkout` step sets
                            `persist-credentials: false`.

                            Failing: any `actions/checkout` step omits
                            `persist-credentials` or sets it to true.
                            """
                        ),
                    ),
                ],
            ),
        ],
    ),
]


OFF_TOPIC_SKILL = r"[\w-]+-rules"

# Should-not-fire: questions no rules skill covers. A rules skill firing here
# is over-triggering. Slugs carry `-neg-` so summarize.py counts them as negatives.
OFF_TOPIC = [
    (
        "neg-mutex-semaphore",
        "in two or three sentences: what's the difference between a mutex and a semaphore?",
        "Passing: the answer says a mutex admits one holder (and is owned by it) while a "
        "semaphore admits up to N concurrent holders by count. Failing: it gets that wrong "
        "or does not answer.",
    ),
    (
        "neg-release-note",
        "draft a two-sentence release note for v2.3 of our notes app. it adds dark mode "
        "and fixes a crash when importing very large markdown files.",
        "Passing: a release note of about two sentences that mentions both dark mode and "
        "the large-markdown import crash fix. Failing: it omits either change or is not "
        "a release note.",
    ),
    (
        "neg-big-o",
        "what's the time complexity of finding the k largest items with a heap of size "
        "k, versus sorting the whole list and slicing?",
        "Passing: the answer gives O(n log k) for the size-k heap and O(n log n) for "
        "sort-then-slice. Failing: either bound is wrong or missing.",
    ),
    (
        "neg-regex-explain",
        "what does this regex match? `^\\d{3}-\\d{4}$`",
        "Passing: the answer says it matches a whole string of exactly three digits, a "
        "hyphen, then four digits (like 555-1234), with nothing before or after. "
        "Failing: it misreads the anchors or the counts.",
    ),
]

