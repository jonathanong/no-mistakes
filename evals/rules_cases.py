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
                "review-empty-per-item",
                """
                review this before i merge

                ```ts
                // test/overdue.test.ts
                import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
                import { overdueInvoices } from "../src/overdue";
                import { seedInvoices } from "./fixtures/invoices";

                describe("overdueInvoices", () => {
                  beforeEach(() => {
                    vi.useFakeTimers();
                    vi.setSystemTime(new Date("2026-03-15T00:00:00Z"));
                  });

                  afterEach(() => {
                    vi.useRealTimers();
                  });

                  it("returns only unpaid invoices past their due date", () => {
                    const result = overdueInvoices(seedInvoices());
                    for (const invoice of result) {
                      expect(invoice.paidAt).toBeNull();
                      expect(invoice.dueDate.getTime()).toBeLessThan(Date.now());
                    }
                  });

                  it("sorts by due date, oldest first", () => {
                    const result = overdueInvoices(seedInvoices());
                    expect(result.map((i) => i.id)).toEqual(["inv_17", "inv_04", "inv_22"]);
                  });

                  it("returns an empty list when everything is paid", () => {
                    const paid = seedInvoices().filter((i) => i.paidAt !== null);
                    expect(overdueInvoices(paid)).toEqual([]);
                  });
                });
                ```

                seedInvoices() is ~200 rows from a fixture file, mixed paid and unpaid.
                """,
                [
                    (
                        "flags-vacuous-loop-not-empty-case",
                        _r(
                            """
                            Grade only the review in the final message. Two checks.

                            1. Passing: the review points at the first test
                            ("returns only unpaid invoices past their due date") and
                            says its per-invoice assertions pass when `result` is
                            empty, so a regression that returns nothing would not
                            fail it; it asks for a non-empty check (or an exact
                            expected list). Failing: the review never raises this,
                            or only says the test is "weak" or "could be stronger"
                            without identifying that an empty result passes.

                            2. Passing: the review does NOT ask to delete the last
                            test (the empty-list case) or to replace its
                            `toEqual([])` with a weaker assertion, and does NOT ask
                            to remove the fake-timer setup and restore. Noting that
                            the last test depends on the fixture data, or that it
                            alone would pass for a function that always returns
                            `[]`, is fine. Failing: it demands deleting or
                            weakening either.

                            Both checks must pass.
                            """,
                            review=True,
                        ),
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
                "write-anti-join",
                """
                findWinbackCustomers(pool): customers who haven't ordered in the
                last 90 days, for a win-back email. node-postgres. guest checkouts
                leave orders.customer_id empty.

                ```sql
                create table customers (
                  id uuid primary key,
                  email text not null,
                  created_at timestamptz not null default now()
                );
                create table orders (
                  id uuid primary key,
                  customer_id uuid references customers(id),
                  created_at timestamptz not null default now()
                );
                create index orders_customer_created_idx on orders (customer_id, created_at);
                ```
                """,
                [
                    (
                        "not-exists-not-not-in",
                        _r(
                            """
                            Grade only the SQL in the final message, and only for how
                            it excludes customers who have recent orders. Other
                            aspects of the query are not part of this grade.

                            Passing: the exclusion uses `NOT EXISTS (...)` (or an
                            equivalent anti-join such as `LEFT JOIN ... WHERE ... IS
                            NULL`).

                            Failing: the exclusion uses `NOT IN (SELECT ...)`, even
                            when the subquery filters out NULL customer ids.
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
        ],
    ),
    (
        "queue-rules",
        [
            (
                "write-enqueue-in-transaction",
                """
                createInvoice(input): insert the invoice row and its line items,
                then queue a send-invoice job with the invoice id so the email goes
                out. helpers below.

                ```ts
                import { db } from "./db"; // db.transaction(async (tx) => ...) commits when the callback resolves, rolls back if it throws
                import { invoiceQueue } from "./queues"; // invoiceQueue.add(name: string, data: object, opts?: { jobId?: string }): Promise<Job>
                import { invoices, invoiceLines } from "./schema"; // tx.insert(table).values(row | rows).returning()

                type Input = {
                  customerId: string;
                  lines: Array<{ sku: string; qty: number; unitCents: number }>;
                };
                ```
                """,
                [
                    (
                        "enqueue-after-commit",
                        _r(
                            """
                            Grade only the code in the final message.

                            Passing: `invoiceQueue.add` is called after the
                            `db.transaction(...)` call has returned (outside the
                            transaction callback), and is awaited.

                            Failing: `invoiceQueue.add` is called inside the
                            transaction callback.
                            """
                        ),
                    ),
                ],
            ),
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
        ],
    ),
    (
        "github-actions-rules",
        [
            (
                "write-path-filtered-required",
                """
                add .github/workflows/backend.yml. run `pnpm --filter api test` on
                PRs to main, node 24. only run it when something under apps/api/ or
                packages/db/ changed, no point burning minutes on frontend PRs.
                "backend / test" is a required check on main.
                """,
                [
                    (
                        "no-path-filter-on-required",
                        _r(
                            """
                            Grade only the workflow in the final message.

                            Passing: the workflow that produces the "backend / test"
                            check starts on every pull request to main (its trigger
                            has no `paths` or `paths-ignore` filter), and any
                            skipping of the tests for unrelated changes is done
                            inside the run, with a job that always reports.

                            Failing: `paths:` or `paths-ignore:` appears under the
                            `pull_request` trigger of that workflow.
                            """
                        ),
                    ),
                ],
            ),
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

