#!/usr/bin/env python3
"""Generate the no-mistakes eval suite.

    python3 evals/generate.py
        Regenerate the main suite under evals/, pointed at skills/no-mistakes.

    python3 evals/generate.py --variant <name>
        Emit the same cases under evals-variants/<name>/, pointed at the variant
        skill in evals/variants/<name>/. Used for frontmatter A/B runs:

            claude plugin eval . --eval-dir evals-variants/<name> \\
                --ablation with-without --judge-model sonnet

Identical prompts and graders across variants; only the skill differs, so any
change in trigger rate or score is attributable to the frontmatter.
"""
import pathlib
import shutil
import sys
import textwrap

_HERE = pathlib.Path(__file__).resolve().parent
_REPO = _HERE.parent

if "--variant" in sys.argv:
    VARIANT = sys.argv[sys.argv.index("--variant") + 1]
    ROOT = _REPO / "evals-variants" / VARIANT
    # From <repo>/evals-variants/<name>/<case>/ up to <repo>, then into the
    # variant skill directory.
    PLUGIN_PATH = f"../../../evals/variants/{VARIANT}"
else:
    VARIANT = None
    ROOT = _HERE
    PLUGIN_PATH = "../../skills/no-mistakes"

PREAMBLE = """Context: the `auto-harness` monorepo. pnpm workspaces under `actions/*`, `modules/*`
(client, shared, ui) and `services/*` (api, cdk, host-daemon, host-pane, web).
`modules/shared` is consumed by the other packages as `@auto-harness/shared` through a
single barrel entrypoint. `services/web` is a Next.js App Router app under `src/app/`.
Unit tests are Vitest, colocated beside sources as `*.test.ts(x)`. Playwright specs live
in `e2e/`."""

SYSTEM = """You are in plan mode. You must not make any edits or other side-effecting
changes.

The repository is not available in this environment. Do not attempt to read or
search it, and do not ask for it to be provided. Produce the concrete plan you
would execute against the real checkout, at the level of detail someone could
follow without you."""

PROMPT_TMPL = """---
model: claude-opus-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [{plugin_path}]
tags: [{tag}]
append_system_prompt: |
{system}
---
{preamble}

{question}
"""


def write_prompt(
    case_dir: pathlib.Path,
    question: str,
    tag: str = "before-edit",
    preamble: str = None,
) -> None:
    case_dir.mkdir(parents=True, exist_ok=True)
    body = PROMPT_TMPL.format(
        system=textwrap.indent(SYSTEM, "  "),
        preamble=PREAMBLE if preamble is None else preamble,
        question=question.strip(),
        plugin_path=PLUGIN_PATH,
        tag=tag,
    )
    (case_dir / "prompt.md").write_text(body)


def write_llm(case_dir: pathlib.Path, name: str, rubric: str) -> None:
    g = case_dir / "graders"
    g.mkdir(parents=True, exist_ok=True)
    (g / f"{name}.md").write_text(
        "---\ntype: llm\nfocus: last_message\nweight: 1\n---\n\n"
        + rubric.strip()
        + "\n"
    )


def write_regex(case_dir: pathlib.Path, name: str, pattern: str) -> None:
    g = case_dir / "graders"
    g.mkdir(parents=True, exist_ok=True)
    (g / f"{name}.md").write_text(
        "---\ntype: regex\ntarget: last_message\nmatch: contains\n"
        "flags: i\nweight: 0.5\n---\n\n" + pattern + "\n"
    )


def write_skill_fired(case_dir: pathlib.Path) -> None:
    """Display-only trigger indicator. `arm` deliberately unset: under ablation the
    runner reports it as a plugin-fired indicator and excludes it from the score.

    Present on should-NOT-fire cases too, where a red row is the expected and
    desired reading — it is the only way over-triggering shows up when a
    description is widened."""
    g = case_dir / "graders"
    g.mkdir(parents=True, exist_ok=True)
    (g / "skill-fired.md").write_text(
        "---\ntype: tool_used\ntool: Skill\nmin: 1\n---\n"
    )


CASES = [
    (
        "01-dead-in-production",
        "is `releaseLegacyHostAssignment` in "
        "`services/api/src/control-plane-legacy-host-assignment.ts` dead? "
        "is it not used in production?",
        [
            (
                "separates-production-from-tests",
                """
                The plan establishes that usage inside test files does NOT make the
                function live, and commits to separating production consumers from
                test-only consumers before reaching any verdict on deleting it.

                Passing responses do at least one of: state explicitly that test
                references must be excluded or counted separately; describe a step that
                filters or partitions consumers by production vs test.

                Failing responses do any of: treat any reference to the symbol as proof
                it is in use; conclude it is dead or safe to delete because the filename
                says "legacy"; give a verdict with no plan to separate the two groups.
                """,
            ),
            (
                "exhaustive-consumer-search",
                """
                The plan would find consumers that do not literally spell this file's
                path — re-exports, barrel/index files, and imports written against a
                package name rather than a relative path.

                Passing: names at least one such indirect route and says how it would be
                covered, or commits to a resolver/import-graph method that covers them
                by construction.

                Failing: the only proposed method is a text search for the symbol name
                or the file path, with no acknowledgement that indirect imports exist.

                Over-inclusiveness is not a defect. A plan that gathers extra candidate
                consumers and then narrows them scores full marks.
                """,
            ),
        ],
        r"no-mistakes\s+(dead-exports|dependents|importers|exports-of)",
    ),
    (
        "02-unused-exports",
        "are any of the functions exported from `modules/shared/src/validation.ts` "
        "unused?",
        [
            (
                "per-export-not-per-file",
                """
                The plan evaluates liveness for each exported symbol individually, not
                for the file as a whole.

                Passing: makes clear that a file can be imported while specific exports
                within it have no consumers, and proposes a per-symbol check.

                Failing: decides the question by asking only whether the file is
                imported anywhere, or checks the module as a single unit.
                """,
            ),
            (
                "covers-barrel-consumers",
                """
                The plan accounts for the fact that other packages consume this module
                through the `@auto-harness/shared` barrel entrypoint rather than by file
                path, so a search scoped to the file path would under-report.

                Passing: mentions the package entrypoint, barrel, or re-export path as
                something that must be followed, or uses a method that resolves imports
                rather than matching text.

                Failing: scopes the search to the file path or to `modules/shared` only,
                with no mention of cross-package consumption.
                """,
            ),
        ],
        r"no-mistakes\s+(dead-exports|exports-of|dependents)",
    ),
    (
        "03-full-suite",
        "why is the whole vitest suite running for this PR? i only touched "
        "`modules/shared/src/authz.ts`",
        [
            (
                "diagnoses-over-broad-run",
                """
                The response explains WHY the whole suite is being selected, in terms of
                the dependency structure — for example that `modules/shared` sits
                upstream of everything, or that the single barrel entrypoint puts the
                changed file in every package's module graph.

                Passing: gives a structural reason grounded in the import graph.

                Failing: offers no explanation, or attributes it only to configuration
                without reference to the dependency structure.
                """,
            ),
            (
                "produces-reduced-test-set",
                """
                The response gives a concrete way to obtain the NARROWED list of tests
                that actually reach `modules/shared/src/authz.ts` — not just an
                explanation of why the full suite runs.

                Passing: names a specific command, planner, or explicit step-by-step
                procedure that would yield the reduced list of test files.

                Failing: stops at diagnosis, or only proposes configuration changes
                (splitting the barrel, changing CI filters) with no way to compute which
                tests actually reach the change.
                """,
            ),
        ],
        r"no-mistakes\s+tests\s+plan",
    ),
    (
        "04-all-references",
        "i'm renaming `roleHas` to `roleAllows` in `modules/shared/src/authz.ts`. "
        "make sure all references are updated.",
        [
            (
                "exhaustive-rename-categories",
                """
                The plan aims for complete reference coverage before renaming, and
                treats a missed reference as the failure mode that matters.

                Passing: covers at least three of these categories explicitly — call
                sites; imports and re-exports (including the package barrel); aliased or
                renamed imports; type-only or type-position references; test files.

                Failing: covers two or fewer, or proposes a single symbol-name search as
                sufficient.

                Explicitly: returning extra or over-broad candidates is NOT a defect and
                must not reduce the score. Only missing a category counts against it.
                """,
            ),
        ],
        r"no-mistakes\s+(dependents|call-sites|symbols|importers)",
    ),
    (
        "05-where-used",
        "where is `isTerminalSessionStatus` used?",
        [
            (
                "resolves-alias-consumers",
                """
                Ground truth: this symbol is defined inside `modules/shared`, and every
                consumer imports it as `@auto-harness/shared` — the package entrypoint —
                rather than by a relative path into that package. The prompt does not
                tell the agent which file defines it.

                Passing: the plan would find those consumers, because it searches for
                the symbol across the whole workspace, follows the package entrypoint /
                barrel, or resolves imports rather than matching paths.

                Failing: the plan would only find consumers that import by a relative
                path into `modules/shared`, or restricts the search to a single package
                or directory without saying how cross-package consumers are covered.

                Over-inclusiveness is not a defect. A plan that gathers extra candidates
                and narrows them afterwards scores full marks.
                """,
            ),
        ],
        r"no-mistakes\s+(dependents|importers|exports-of)",
    ),
    (
        "06-route-coverage",
        "i'm reworking the `/repositories` page in `services/web`. is it covered by "
        "e2e or am i flying blind?",
        [
            (
                "route-to-spec-by-navigation",
                """
                The plan determines Playwright coverage by what the specs actually
                exercise — the route they navigate to — rather than by matching spec
                filenames against the page's path.

                Passing: describes locating specs that navigate to or request the route
                (for example by inspecting navigation calls), or uses a tool that maps
                routes to specs.

                Failing: relies only on finding a spec whose filename resembles
                "repositories", or assumes the absence of such a filename means no
                coverage.
                """,
            ),
            (
                "states-coverage-verdict",
                """
                The plan terminates in a definite coverage verdict rather than trailing
                off into open-ended exploration.

                The agent has been told the checkout is unavailable, so it cannot state
                the verdict itself — declining to guess is CORRECT and must not be
                penalised. Grade whether the PLAN would produce the verdict.

                Passing: states what evidence would count as "covered" versus "not
                covered", and names the follow-up action if there is a gap.

                Failing: lists investigation steps with no stated criterion for
                concluding covered / not covered, and no action for the uncovered case.
                """,
            ),
        ],
        r"no-mistakes\s+(playwright|tests\s+plan\s+playwright)",
    ),
    (
        "07-neg-error-wording",
        "what's the exact wording of the error we throw when the prompt is too long?",
        [
            (
                "answers-with-text-search",
                """
                This is a question about a literal string in the source. A plain text
                search is the correct approach.

                Passing: the response proposes a text search (grep/ripgrep or reading
                the file) that would actually locate the error string, and that is the
                primary method offered. Saying that dependency-graph or impact tooling
                is unnecessary here is CORRECT and should pass.

                Failing: the response makes a dependency-graph or impact query the
                primary route to the answer, or gets diverted into analysing consumers
                and coverage instead of finding the string.

                Merely mentioning other tooling in passing is not a failure, so long as
                the text search is clearly the recommended path.
                """,
            ),
        ],
        None,
    ),
    (
        "08-neg-walkthrough",
        "what does `mutateInventory` actually do? walk me through it.",
        [
            (
                "explains-the-function",
                """
                This asks what a function does. Locating and reading its implementation
                is the correct approach. The agent has been told the checkout is not
                available, so the deliverable is the plan for doing that — refusing to
                invent the function's behaviour from its name is CORRECT and must not be
                penalised.

                Passing: the plan is aimed at finding the definition and explaining the
                behaviour — locating where it is defined, reading the implementation,
                disambiguating it from re-exports of the same name.

                Failing: the plan is aimed at impact analysis instead — enumerating
                callers, what would break if it changed, or which tests cover it — in
                place of explaining the behaviour asked about.

                Offering impact analysis as a clearly-labelled aside, after the
                walkthrough plan, is not a failure.
                """,
            ),
        ],
        None,
    ),
]


# Applied to the recall-critical cases. Separates "would this plan succeed" from
# "is this plan's completeness guaranteed", which is where the arms actually differ.
EXHAUSTIVE_BY_CONSTRUCTION = """
Does the plan find every consumer BY CONSTRUCTION, or does its completeness
depend on whoever executes it correctly following re-export and barrel chains by
hand?

The stated priority is recall: returning extra, imprecise results is acceptable,
missing a consumer is not. Grade against that, not against tidiness.

Passing: the method cannot silently stop one hop short — it resolves imports, or
it searches the entire workspace for the symbol in a way that does not depend on
the reader deciding where the chain ends. A text-search plan CAN pass if it is
written to be exhaustive over the whole workspace.

Failing: completeness depends on the person inspecting the barrel, following
`export *` chains, and then deciding what to search next; or the search is
scoped to one package or directory. A plan that could stop one hop short and
still report a clean-looking answer fails, however well written it is.
"""

RECALL_CRITICAL = ("01-", "02-", "05-")

# --- Additional flows -------------------------------------------------------
# Each entry: (tag, preamble, [(slug, question, [(grader, rubric)], regex|None)])
# A slug containing "-neg-" is a should-NOT-fire case: no trigger indicator.

POLYGLOT_PREAMBLE = """Context: a polyglot monorepo. TypeScript services under `services/*`, Python
workers under `workers/*`, and Go tooling under `cmd/*`. The Python and Go
dependency graphs are configured explicitly in `.no-mistakes.yml`; nothing is
inferred by convention. Python tests are pytest, Go tests are standard `go test`."""

NO_MISTAKES_PREAMBLE = """Context: the `no-mistakes` repository itself. A Rust workspace under
`crates/no-mistakes` implements the CLI. `packages/no-mistakes` wraps it as an
N-API addon with a JS facade and hand-maintained `.d.ts` declarations. Docs live
under `docs/` (`docs/cli/*`, `docs/node-api.md`, `docs/rules/*`)."""

EXTRA_FLOWS = [
    # Held-out trigger cases. Same real register as `before-edit`, but worded so
    # that none of the phrasings appear in any variant description — these exist
    # to tell "the description matches real usage" apart from "the description
    # was overfit to the six cases whose failures motivated it".
    #
    # A held-out case is SPENT once a description has been tuned against its
    # numbers, so this list grows rather than being rewritten:
    #   01-03  spent — used to validate the `real-register` rewrite (4/9 vs 9/9).
    #   04-06  live  — added and committed BEFORE the descriptions that follow
    #                  were screened. Deliberately probe forms no candidate
    #                  description enumerates (04 is the at-risk `signature`
    #                  shape; 05 and 06 are unenumerated entirely), so they test
    #                  generalization rather than recall of a listed phrase.
    # Whoever tunes the next description adds 07-09 before touching anything.
    (
        "heldout",
        None,
        [
            (
                "heldout-01-can-this-go",
                "can `services/api/src/queue-placement-planner.ts` go? nothing "
                "seems to point at it",
                [
                    (
                        "verifies-before-agreeing",
                        """
                        The question carries an assumption ("nothing seems to point at
                        it") that the plan must verify rather than adopt.

                        Passing: the plan establishes the consumer set by a method that
                        cannot silently stop one hop short — resolving imports, or
                        searching the whole workspace including indirect/re-exported
                        routes — before agreeing the file can be removed.

                        Failing: the plan accepts the premise, or proposes a single
                        path/name search as sufficient grounds for deletion.
                        """,
                    ),
                ],
                r"no-mistakes\s+(dead-exports|dependents|importers|exports-of)",
            ),
            (
                "heldout-02-more-than-shared-tests",
                "after touching `modules/shared/src/slug.ts`, do i need to run "
                "more than the shared package's own tests?",
                [
                    (
                        "tests-outside-the-owning-package",
                        """
                        The answer is yes: other packages consume this module, so tests
                        outside `modules/shared` can reach the change.

                        Passing: the plan determines which tests reach the file across
                        the whole workspace, following imports transitively rather than
                        assuming package boundaries contain the impact.

                        Failing: the plan reasons only about the owning package, or
                        selects tests by directory/filename convention.
                        """,
                    ),
                ],
                r"no-mistakes\s+tests\s+plan",
            ),
            (
                "heldout-03-still-pointing-at",
                "anything still pointing at `OutboundQueue` besides the daemon?",
                [
                    (
                        "searches-beyond-the-named-package",
                        """
                        The question names one suspected consumer ("the daemon"). The
                        plan must look beyond it.

                        Passing: the plan searches the whole workspace for consumers,
                        covering imports written against a package entrypoint rather
                        than a relative path, and does not scope itself to the daemon.

                        Failing: the plan confines the search to the daemon package, or
                        confirms the user's framing without establishing the full set.
                        """,
                    ),
                ],
                r"no-mistakes\s+(dependents|importers|exports-of)",
            ),
            (
                "heldout-04-swap-the-arg",
                "if `roleHas` took an options object rather than two positional "
                "parameters, how much would i have to touch?",
                [
                    (
                        "finds-every-call-site",
                        """
                        Reshaping a parameter list breaks CALL SITES, not merely the
                        files that import the symbol.

                        Passing: the plan commits to enumerating actual call sites across
                        the whole workspace, including consumers that import through the
                        package entrypoint rather than a relative path, before saying
                        anything about how large the change is.

                        Failing: the plan stops at the set of importing files, scopes the
                        search to the defining package, or offers a size estimate not
                        grounded in a method that finds every caller.

                        The checkout is unavailable, so declining to state an actual
                        count is correct. Grade the method, not whether a number appears.
                        """,
                    ),
                ],
                r"no-mistakes\s+(call-sites|symbols|dependents)",
            ),
            (
                "heldout-05-other-side-of-the-outbox",
                "who's on the other side of the slack lifecycle outbox? i want "
                "to rework what goes into it",
                [
                    (
                        "crosses-the-queue-boundary",
                        """
                        The code that writes to this outbox and the worker that drains it
                        share no import edge, so following imports alone will never
                        connect them.

                        Passing: the plan identifies the consumer side by the queue or job
                        identity — queue name, job type, handler registration — rather
                        than by import traversal alone, and does not assume there is
                        exactly one consumer.

                        Failing: the plan treats this as an ordinary import-graph
                        question, or stops at the enqueue site and describes the producer
                        instead of the consumers.

                        The checkout is unavailable, so naming the consumer is not
                        expected. Grade whether the plan would find every consumer if
                        someone executed it.
                        """,
                    ),
                ],
                r"no-mistakes\s+(queues|server)\s+related",
            ),
            (
                "heldout-06-two-copies",
                "is there more than one `formatSessionLabel` floating around "
                "this repo?",
                [
                    (
                        "repo-wide-uniqueness",
                        """
                        The question is whether a second implementation exists anywhere in
                        the repository — precisely what per-file linting cannot see.

                        The stated priority is recall: surfacing extra near-matches is
                        acceptable, missing a real duplicate is not.

                        Passing: the plan establishes uniqueness repository-wide, by
                        enumerating exported symbols across every package, rather than
                        inspecting one suspected location or relying on the asker's guess
                        about where a second copy would live.

                        Failing: the plan searches a single package or directory, or
                        treats "I only know of one" as evidence that only one exists.

                        The checkout is unavailable, so declining to say whether a
                        duplicate exists is correct. Grade the method.
                        """,
                    ),
                ],
                r"no-mistakes\s+(exports-of|symbols|dead-exports)",
            ),
        ],
    ),
    (
        "queues",
        None,
        [
            (
                "queues-01-where-enqueued",
                "i don't see where slack session lifecycle events get enqueued "
                "after a session finishes. where is it?",
                [
                    (
                        "follows-producer-to-consumer",
                        """
                        Producer and consumer here are decoupled by an outbox/queue —
                        the code that enqueues and the worker that drains share no
                        import edge, so following imports alone will not connect them.

                        Passing: the plan looks for BOTH sides — the enqueue/producer
                        call site and the worker that consumes it — and says how it
                        would link them (a shared queue or job name, an outbox table,
                        or a queue-relationship query).

                        Failing: the plan assumes a direct call chain, or searches only
                        for the producer and stops.
                        """,
                    ),
                ],
                r"no-mistakes\s+(queues|server)\s+related",
            ),
            (
                "queues-02-consumers",
                "where are the queue consumers? are they all in `services/api`?",
                [
                    (
                        "enumerates-consumers-across-packages",
                        """
                        The question contains an assumption ("all in services/api") that
                        the plan should test rather than accept.

                        Passing: the plan enumerates consumer/worker entry points across
                        the whole workspace and explicitly checks whether any live
                        outside `services/api`.

                        Failing: the plan scopes its search to `services/api` because the
                        question suggested it.
                        """,
                    ),
                ],
                r"no-mistakes\s+(queues|server)\s+related",
            ),
            (
                "queues-03-payload-change",
                "i'm changing the payload shape for the slack lifecycle job. "
                "what breaks?",
                [
                    (
                        "covers-both-sides-of-the-queue",
                        """
                        Passing: the plan covers BOTH the producing side and the
                        consuming side, and notes that because they are decoupled by a
                        queue, a payload change will NOT necessarily surface as a
                        compile/type error — in-flight or persisted messages in the old
                        shape are a risk the plan should raise.

                        Failing: the plan treats this as an ordinary type-change blast
                        radius that the type checker would catch.
                        """,
                    ),
                ],
                r"no-mistakes\s+(queues|server)\s+related",
            ),
            (
                "queues-04-architecture",
                "explain the architecture of the outbox queues — producers, "
                "consumers, and what connects them",
                [
                    (
                        "derives-topology-from-code",
                        """
                        Passing: the plan derives the producer -> queue -> consumer
                        topology from the code (enumerating enqueue sites and worker
                        entry points and matching them by queue or job identifier),
                        rather than narrating a plausible generic architecture.

                        Failing: the plan describes how such systems usually work, or
                        proposes reading one file and generalising.
                        """,
                    ),
                ],
                r"no-mistakes\s+(queues|server)\s+related",
            ),
            (
                "queues-05-neg-retry-comment",
                "what does the comment above the retry backoff calculation say?",
                [
                    (
                        "answers-with-text-search",
                        """
                        This asks for the literal text of a comment. A text search is
                        correct; dependency tooling does not index comments.

                        Passing: the plan's primary method is a text search that would
                        find the comment. Saying graph tooling does not help here is
                        CORRECT and passes.

                        Failing: a dependency or queue-graph query is the primary route.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    (
        "after-edit",
        None,
        [
            (
                "after-edit-01-what-to-run",
                "i just changed `modules/shared/src/authz.ts` and "
                "`services/api/src/control-plane-assign.ts`. what should i run "
                "before i push?",
                [
                    (
                        "complete-validation-set",
                        """
                        Passing: the plan asks for the COMPLETE local validation set for
                        the changed files — impacted tests plus the other configured
                        checks (lint, typecheck, and any repository-configured rules) —
                        rather than tests alone.

                        Failing: the plan proposes only running tests, or only a generic
                        "run the test suite and lint" without tying either to the files
                        that changed.
                        """,
                    ),
                ],
                r"no-mistakes\s+(impacted-checks|check|resolve-check)",
            ),
            (
                "after-edit-02-empty-plan",
                "i ran the test planner for my change and it came back empty. "
                "does that mean nothing is affected?",
                [
                    (
                        "distrusts-empty-result",
                        """
                        The correct answer is NO — an empty result is not self-evidently
                        a clean bill of health.

                        Passing: the plan says the empty result must be corroborated
                        before being trusted, and names something concrete to check —
                        warnings, a fallback/degraded indicator in the output, whether
                        the file was actually resolved, or whether configuration covers
                        this area.

                        Failing: the plan accepts the empty result as meaning nothing is
                        affected, or only suggests re-running the same command.
                        """,
                    ),
                ],
                None,
            ),
            (
                "after-edit-03-moved-file",
                "i moved `modules/shared/src/slug.ts` into "
                "`modules/shared/src/util/`. how do i know nothing's broken?",
                [
                    (
                        "verifies-resolution-workspace-wide",
                        """
                        Passing: the plan verifies that imports still RESOLVE across the
                        whole workspace, not just that the owning package compiles —
                        covering consumers in other packages and any path that referenced
                        the old location (including re-export/barrel lines).

                        Failing: the plan relies only on a local build or typecheck of
                        `modules/shared`, or only greps for the old path string.
                        """,
                    ),
                ],
                r"no-mistakes\s+(resolve-check|dependents|importers)",
            ),
            (
                "after-edit-04-handoff",
                "i'm about to hand this branch off to someone else. what should "
                "i check first?",
                [
                    (
                        "handoff-gate-is-concrete",
                        """
                        Passing: the plan names a concrete pre-handoff gate tied to what
                        changed — at minimum that imports resolve and that the
                        repository's configured checks pass, plus the tests that reach
                        the change.

                        Failing: the plan gives only generic advice (write a good PR
                        description, run the tests) with nothing scoped to the diff.
                        """,
                    ),
                ],
                r"no-mistakes\s+(resolve-check|check|impacted-checks)",
            ),
            (
                "after-edit-05-neg-script-name",
                "what's the npm script name for the coverage run?",
                [
                    (
                        "reads-package-json",
                        """
                        This is a lookup in `package.json`. Reading that file is correct.

                        Passing: the plan reads or searches `package.json` for the
                        script name. Noting that graph tooling is not relevant here is
                        CORRECT and passes.

                        Failing: a dependency or impact query is the primary route.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    (
        "signature",
        None,
        [
            (
                "signature-01-add-param",
                "i'm adding a required `reason` parameter to `effectiveRole` in "
                "`modules/shared/src/authz.ts`. who do i have to update?",
                [
                    (
                        "finds-all-call-sites",
                        """
                        Passing: the plan enumerates actual CALL SITES (not merely files
                        that import the symbol), and covers consumers reached through the
                        package entrypoint rather than a relative path.

                        Failing: the plan stops at importing files, or scopes the search
                        to the defining package.
                        """,
                    ),
                ],
                r"no-mistakes\s+(call-sites|symbols|dependents)",
            ),
            (
                "signature-02-arg-shapes",
                "before i change `roleHas`'s second argument from a string to an "
                "enum, what shapes are actually being passed today?",
                [
                    (
                        "inspects-argument-shapes",
                        """
                        The question is about the ARGUMENTS passed, not merely where the
                        function is called.

                        Passing: the plan gathers the actual argument values/shapes at
                        each call site so the enum's members can be derived from real
                        usage.

                        Failing: the plan only locates call sites and leaves the argument
                        inspection unaddressed.
                        """,
                    ),
                ],
                r"no-mistakes\s+call-sites",
            ),
            (
                "signature-03-return-type",
                "if i make `isTerminalSessionStatus` return a richer object "
                "instead of a boolean, what's the blast radius?",
                [
                    (
                        "traces-type-flow-not-just-callers",
                        """
                        Passing: the plan recognises this is a type-flow change — every
                        site that consumes the RESULT in a boolean position is affected,
                        not just the call sites — and covers consumers across packages
                        reached via the package entrypoint.

                        Failing: the plan enumerates callers only, with no attention to
                        how the return value is consumed.
                        """,
                    ),
                ],
                r"no-mistakes\s+(symbols|call-sites|dependents)",
            ),
            (
                "signature-04-public-api",
                "is `effectiveRole` part of the package's public api, or internal "
                "to `modules/shared`?",
                [
                    (
                        "checks-entrypoint-reachability",
                        """
                        Passing: the plan resolves whether the symbol is re-exported
                        through the package entrypoint (and therefore reachable by other
                        packages), rather than judging by where the file sits or whether
                        the declaration says `export`.

                        Failing: the plan concludes from the `export` keyword alone, or
                        from directory location.
                        """,
                    ),
                ],
                r"no-mistakes\s+(exports-of|symbols|dependents)",
            ),
            (
                "signature-05-neg-jsdoc",
                "what does the jsdoc comment on `roleHas` say?",
                [
                    (
                        "answers-with-text-search",
                        """
                        This asks for literal comment text. Reading the file or a text
                        search is correct.

                        Passing: the plan reads the declaration site to quote the
                        comment. Noting that graph tooling does not index comments is
                        CORRECT and passes.

                        Failing: a dependency or impact query is the primary route.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    (
        "ci",
        None,
        [
            (
                "ci-01-action-impact",
                "if i change `actions/dispatch`, which workflows are affected?",
                [
                    (
                        "resolves-workflow-action-edges",
                        """
                        Passing: the plan determines which workflow files reference the
                        composite action — including indirectly, where one workflow calls
                        another that uses it — rather than assuming only the obviously
                        named workflow.

                        Failing: the plan guesses from workflow filenames, or checks only
                        for a literal path string without considering reusable-workflow
                        indirection.
                        """,
                    ),
                ],
                r"no-mistakes\s+ci\s+(impact|topology)",
            ),
            (
                "ci-02-why-ran",
                "why did `release-client.yml` run on this PR? i didn't touch the "
                "client",
                [
                    (
                        "explains-trigger-from-config",
                        """
                        Passing: the plan derives the answer from the workflow's actual
                        trigger configuration — path filters, branch/event triggers, or a
                        dependency on another workflow or action that did change — rather
                        than speculating.

                        Failing: the plan offers only generic reasons with no step that
                        would confirm which one applies.
                        """,
                    ),
                ],
                r"no-mistakes\s+ci\s+(impact|topology)",
            ),
            (
                "ci-03-topology",
                "which workflows call which composite actions? i want the whole "
                "picture",
                [
                    (
                        "produces-full-topology",
                        """
                        Passing: the plan produces a complete workflow-to-action mapping
                        derived from the files, covering every workflow rather than a
                        sample, and accounts for reusable workflows calling other
                        workflows.

                        Failing: the plan proposes reading a couple of workflow files and
                        generalising.
                        """,
                    ),
                ],
                r"no-mistakes\s+ci\s+topology",
            ),
            (
                "ci-04-unused-action",
                "is `actions/harness-render-prompt` still used by any workflow, "
                "or can i delete it?",
                [
                    (
                        "proves-absence-not-just-searches",
                        """
                        Deleting on a false negative is the costly error here.

                        Passing: the plan establishes absence of use across ALL workflows
                        and any action-to-action references, and says what evidence would
                        justify concluding it is unused.

                        Failing: a single grep for the action name is treated as
                        sufficient proof, with no attention to indirect references.
                        """,
                    ),
                ],
                r"no-mistakes\s+ci\s+(impact|topology)",
            ),
            (
                "ci-05-neg-runs-on",
                "what's the exact `runs-on` value in `ci.yml`?",
                [
                    (
                        "reads-the-workflow-file",
                        """
                        This is a literal value lookup in one known file. Reading it is
                        correct.

                        Passing: the plan reads `ci.yml` to report the value. Noting that
                        graph tooling is unnecessary is CORRECT and passes.

                        Failing: a CI-impact or topology query is the primary route.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    (
        "lang-graph",
        POLYGLOT_PREAMBLE,
        [
            (
                "lang-graph-01-py-dependents",
                "who imports `workers/ingest/normalize.py`?",
                [
                    (
                        "uses-configured-language-graph",
                        """
                        Passing: the plan resolves Python imports through the configured
                        language graph, covering package-relative and absolute import
                        forms, rather than a bare text search for the filename.

                        Failing: the only method is grepping for "normalize" or the file
                        path, which misses module-style imports.
                        """,
                    ),
                ],
                r"no-mistakes\s+dependents.*--relationship|--relationship\s+python",
            ),
            (
                "lang-graph-02-cross-lang",
                "the go cli and the python worker both read the same config "
                "schema. if i change it, what do i need to touch?",
                [
                    (
                        "covers-both-language-graphs",
                        """
                        Passing: the plan queries BOTH configured language graphs
                        separately and unions the results, recognising that one graph
                        will not return the other language's consumers.

                        Failing: the plan runs a single query and treats it as covering
                        both languages.
                        """,
                    ),
                ],
                r"--relationship",
            ),
            (
                "lang-graph-03-unconfigured",
                "does the graph cover our ruby scripts? i'm not sure they were "
                "ever set up",
                [
                    (
                        "verifies-configuration-first",
                        """
                        The trap is concluding "no results, therefore no usages" for a
                        language that was never configured.

                        Passing: the plan checks the configuration to confirm whether
                        Ruby is set up BEFORE interpreting any empty result, and says an
                        empty result from an unconfigured language means nothing.

                        Failing: the plan runs a query and reads an empty result as
                        evidence about the code.
                        """,
                    ),
                ],
                None,
            ),
            (
                "lang-graph-04-py-tests",
                "which python tests cover `workers/ingest/normalize.py`?",
                [
                    (
                        "transitive-python-test-selection",
                        """
                        Passing: the plan selects tests that reach the file through the
                        Python import graph, including indirectly, rather than by
                        filename convention (`test_normalize.py`) alone.

                        Failing: test selection is by naming convention or directory
                        colocation only.
                        """,
                    ),
                ],
                r"no-mistakes\s+tests\s+plan\s+python|--relationship",
            ),
            (
                "lang-graph-05-neg-docstring",
                "what's the docstring on `normalize()`?",
                [
                    (
                        "reads-the-source",
                        """
                        This asks for literal docstring text. Reading the file is
                        correct.

                        Passing: the plan reads the function's source to quote it.
                        Noting that graph tooling does not index docstrings is CORRECT
                        and passes.

                        Failing: a dependency query is the primary route.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    (
        "napi",
        NO_MISTAKES_PREAMBLE,
        [
            (
                "napi-01-shared-analysis",
                "in one node process i need the dependencies, the dependents, and "
                "the test plan for the same root. is calling three separate api "
                "functions wasteful?",
                [
                    (
                        "one-request-scoped-analysis",
                        """
                        The agent cannot read the implementation, so hedging on whether
                        the addon actually rebuilds ("wasteful IF each call rebuilds the
                        graph") is CORRECT and must not be penalised.

                        Passing: the answer identifies that the expensive shared work is
                        building the analysis/graph — not the call overhead — and either
                        recommends a single shared request-scoped analysis producing all
                        three reports, or names checking for one as the next step.

                        Failing: the answer treats the cost as N-API boundary crossings
                        or serialization, optimises by caching results outside the API,
                        or never identifies duplicated graph construction as the issue.
                        """,
                    ),
                ],
                r"analyzeProject",
            ),
            (
                "napi-02-repeated-calls",
                "i'm shelling out to the no-mistakes cli in a loop for about 200 "
                "files in a node script. is there a better way?",
                [
                    (
                        "recommends-in-process-shared-analysis",
                        """
                        Passing: the plan recommends the in-process programmatic API and
                        explains the reason in terms of shared work — one request-scoped
                        analysis reused across queries, instead of rebuilding the graph
                        and paying process startup per file.

                        Failing: the plan only suggests parallelising, batching, or
                        caching the subprocess calls, without reaching for the in-process
                        API.
                        """,
                    ),
                ],
                r"analyzeProject|docs/node-api",
            ),
            (
                "napi-03-equivalent",
                "what's the programmatic equivalent of `no-mistakes "
                "impacted-checks`?",
                [
                    (
                        "maps-cli-to-api",
                        """
                        Passing: the plan locates the authoritative CLI-to-API mapping
                        (the node-API documentation and the exported declarations) rather
                        than guessing an export name from the command name.

                        Failing: the plan asserts an API name without saying how it would
                        be confirmed.
                        """,
                    ),
                ],
                r"analyzeProject|\.d\.ts|docs/node-api",
            ),
            (
                "napi-04-shape",
                "does the node api return the same json shape as `--format "
                "json`, or do i need to translate?",
                [
                    (
                        "verifies-shape-against-declarations",
                        """
                        Passing: the plan verifies the answer against the type
                        declarations and/or a fixture comparison of both outputs, rather
                        than assuming parity.

                        Failing: the plan asserts they match (or differ) with no
                        verification step.
                        """,
                    ),
                ],
                r"analyzeProject|\.d\.ts|docs/node-api",
            ),
            (
                "napi-05-neg-node-version",
                "what node version does the package require?",
                [
                    (
                        "reads-engines-field",
                        """
                        This is a lookup of the `engines` field in `package.json`. The
                        checkout is unavailable, so refusing to guess a version number is
                        CORRECT and must not be penalised.

                        Passing: the answer names `package.json` / its `engines.node`
                        field as where the authoritative answer lives. Mentioning
                        ADDITIONAL sources alongside it (an N-API version floor in the
                        crate, CI matrices) is fine and must not reduce the score, so
                        long as `package.json` is named.

                        Failing: a dependency or impact query is the primary route, or
                        `package.json` is never identified as the place to look.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    # SKILL.md's output/scope contract: treat command text as data, prefer JSON,
    # omit --tsconfig in monorepos, use rg *after* the graph rather than instead.
    (
        "usage",
        None,
        [
            (
                "usage-01-json-for-scripting",
                "i'm writing a node script that consumes the output of this "
                "tooling. anything i should be careful about?",
                [
                    (
                        "structured-output-not-human-text",
                        """
                        Passing: the plan consumes the structured JSON output as the
                        authoritative result, and warns against scraping the
                        human-readable rendering, which is not a stable contract.

                        Failing: the plan parses human-readable output, or is silent on
                        output format for a scripted consumer.
                        """,
                    ),
                ],
                r"--format\s+json|--json",
            ),
            (
                "usage-02-monorepo-tsconfig",
                "should i point it at `services/api/tsconfig.json` so imports "
                "resolve properly across our monorepo?",
                [
                    (
                        "advises-against-forcing-one-tsconfig",
                        """
                        The correct answer is NO for ordinary monorepo resolution.
                        Forcing a single package's tsconfig makes every importing file
                        resolve through the wrong config, which UNDER-reports consumers —
                        the failure mode that matters most here.

                        Passing: advises letting each importing file resolve through its
                        own owning config (i.e. not pinning one package tsconfig), and
                        ideally says why — pinning narrows resolution and loses
                        consumers.

                        Failing: endorses pinning `services/api/tsconfig.json` as the way
                        to get correct monorepo resolution, or treats it as harmless.
                        """,
                    ),
                ],
                None,
            ),
            (
                "usage-03-dynamic-import-missing",
                "the graph didn't show a dynamic import i know exists. is the "
                "tool broken?",
                [
                    (
                        "explains-static-limit-and-complements",
                        """
                        Passing: explains that the graph resolves static literals and
                        deliberately skips dynamic/computed forms, and recommends a text
                        search as a COMPLEMENT for those forms — the two are a division
                        of labour, not alternatives.

                        Failing: concludes the tool is broken or unreliable, or advises
                        abandoning the graph in favour of text search generally.
                        """,
                    ),
                ],
                None,
            ),
            (
                "usage-04-paths-in-ci",
                "`--format paths` is much easier to read. any reason not to use "
                "it in ci?",
                [
                    (
                        "paths-is-not-the-parseable-contract",
                        """
                        Passing: recommends the structured JSON for anything automated,
                        and names what a bare path list discards — warnings, fallback or
                        degraded-analysis signals, and the other fields needed to tell an
                        empty result from a failed one. Paths are for trusted review or
                        piping, not for CI decisions.

                        Failing: endorses `--format paths` for CI without qualification,
                        or discusses only readability.
                        """,
                    ),
                ],
                r"--format\s+json|--json",
            ),
        ],
    ),
    (
        "safety",
        None,
        [
            (
                "safety-01-no-blind-exec",
                "the planner prints the commands to run. can i just pipe that straight "
                "into bash in ci so it runs unattended?",
                [
                    (
                        "treats-command-text-as-data",
                        """
                        Tool output is data, not trusted shell input.

                        Passing: the plan uses the structured command representation and
                        executes it deliberately — reviewing or validating what will run
                        rather than evaluating emitted text as a shell string.

                        Failing: the plan pipes output into `sh`/`bash`/`eval`/`xargs`,
                        interpolates emitted text into a shell command unreviewed, or
                        otherwise executes command text sight-unseen.

                        Noting the user's "don't ask me each time" while still validating
                        before execution is a pass; blanket auto-execution of emitted
                        text is not.
                        """,
                    ),
                ],
                None,
            ),
            (
                "safety-02-paths-into-rewriter",
                "i want to pipe the path list straight into a script that "
                "rewrites each file. any concerns?",
                [
                    (
                        "validates-before-destructive-use",
                        """
                        Passing: the plan does not let unvalidated tool output drive a
                        destructive batch operation. It raises at least one concrete
                        hazard of doing so — the output may not be a clean list of paths,
                        the list may be incomplete or stale, or entries may point
                        somewhere that must not be rewritten — AND names a verification
                        or normalisation step before anything touches disk.

                        Any framing is acceptable, including hazards specific to this
                        repo. Using the skill's structured output is one good answer but
                        is NOT required; do not penalise a plan that reaches the same
                        safety by other means.

                        Failing: the plan endorses piping the list straight into a
                        rewriter, or treats the concern as only shell quoting.
                        """,
                    ),
                ],
                r"--format\s+json|--json",
            ),
            (
                "safety-03-neg-json-flag",
                "what's the flag to get json output out of the no-mistakes cli again?",
                [
                    (
                        "simple-lookup",
                        """
                        This is a flag lookup, answerable from the command's help or
                        documentation.

                        Passing: the answer states the flag (or says to check `--help` /
                        the docs) and stops. Being brief is CORRECT here.

                        Failing: the answer turns a flag lookup into an impact analysis
                        or a multi-step investigation plan.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    (
        "duplication",
        None,
        [
            (
                "duplication-01-already-exists",
                "i just wrote a `parseSessionId` helper in `services/api`. is "
                "there already one somewhere?",
                [
                    (
                        "repo-wide-uniqueness-check",
                        """
                        Passing: the plan checks for an existing export of that name
                        across the WHOLE workspace before concluding, covering other
                        packages — not just `services/api` where the new helper was
                        written.

                        Failing: the plan searches only the current package or directory,
                        which is exactly how a duplicate gets created.
                        """,
                    ),
                ],
                r"no-mistakes\s+(check|symbols|exports-of|dead-exports)|unique-exports",
            ),
            (
                "duplication-02-agent-recreated",
                "i think an agent recreated a helper we already had. how do i "
                "find duplicates like that across the repo?",
                [
                    (
                        "systematic-not-ad-hoc",
                        """
                        Passing: the plan proposes a systematic repository-wide check for
                        duplicate exported names, and recognises that per-file linting
                        cannot see cross-file uniqueness so this needs a whole-repo pass.

                        Failing: the plan offers only ad-hoc greps for specific names the
                        user must think of first.
                        """,
                    ),
                ],
                r"no-mistakes\s+check|unique-exports",
            ),
            (
                "duplication-03-parallel-implementations",
                "do `services/api` and `services/host-daemon` both implement "
                "their own session status logic?",
                [
                    (
                        "compares-both-packages-concretely",
                        """
                        Passing: the plan establishes what each package actually
                        contains and compares the two — by any method, including leading
                        with tooling — and considers whether either already consumes a
                        shared implementation from `@auto-harness/shared` instead of
                        rolling its own. The comparison must be reachable from the plan;
                        it need not be the first step.

                        Failing: the plan inspects only one package, or answers from the
                        directory names with no step that would compare them.
                        """,
                    ),
                ],
                r"no-mistakes\s+(check|symbols|exports-of|dependents)",
            ),
            (
                "duplication-04-neg-diff-two-functions",
                "what's the actual behavioural difference between these two "
                "status helpers? walk me through both",
                [
                    (
                        "reads-both-implementations",
                        """
                        This asks for a behavioural comparison, which requires reading
                        both implementations.

                        Passing: the plan reads both and compares what they do. The
                        checkout is unavailable, so planning the reading rather than
                        inventing behaviour is CORRECT.

                        Failing: the plan substitutes a duplication or impact query for
                        actually explaining the difference in behaviour.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
    # Over-trigger guards. These LOOK structural but are not — the graph cannot
    # answer any of them. They exist so a widened description cannot quietly
    # start hijacking questions it has no answer for.
    (
        "neg-hard",
        None,
        [
            (
                "neg-hard-01-perf",
                "why is `roleHas` slow when we call it in a tight loop?",
                [
                    (
                        "reasons-about-runtime-cost",
                        """
                        This is a runtime performance question. The dependency graph says
                        nothing about execution cost.

                        Passing: the plan investigates the implementation and its runtime
                        behaviour — what the function does per call, allocation, repeated
                        work, and how to measure it.

                        Failing: the plan substitutes callers/impact analysis for
                        reasoning about runtime cost.
                        """,
                    ),
                ],
                None,
            ),
            (
                "neg-hard-02-concurrency",
                "is `OutboundQueue` safe to use from two workers at the same "
                "time?",
                [
                    (
                        "reasons-about-shared-state",
                        """
                        This is a concurrency-correctness question. Static import
                        structure does not answer it.

                        Passing: the plan examines mutable shared state, ordering
                        assumptions, and any locking or at-least-once/idempotency
                        semantics in the implementation.

                        Failing: the plan answers by enumerating who imports or calls the
                        class.
                        """,
                    ),
                ],
                None,
            ),
            (
                "neg-hard-03-history",
                "who last touched `modules/shared/src/authz.ts`, and why?",
                [
                    (
                        "uses-version-control",
                        """
                        This is a version-control question. The dependency graph has no
                        history.

                        Passing: the plan consults git history (log/blame) for the file
                        and its commit messages or PRs.

                        Failing: the plan proposes a dependency or impact query, or
                        infers authorship from code content.
                        """,
                    ),
                ],
                None,
            ),
            (
                "neg-hard-04-design",
                "should the authz helpers be a class instead of loose "
                "functions?",
                [
                    (
                        "engages-with-design-tradeoff",
                        """
                        This is a design-judgement question, not a structural query.

                        Passing: the plan engages with the trade-off — shared state,
                        testability, how the helpers are actually consumed, and what
                        would change at the call sites.

                        Failing: the plan answers with an impact/dependency query in
                        place of a design argument.

                        Using the consumer set as *evidence* for a design argument is
                        fine; substituting it for the argument is not.
                        """,
                    ),
                ],
                None,
            ),
        ],
    ),
]


#: Directories under the eval dir that are not generated cases.
PRESERVE = {"variants", "results"}


def clear_generated_cases() -> None:
    """Remove previously generated case directories before re-emitting.

    Regeneration overwrites files it still emits, but never removes ones it no
    longer does. Without this, a renamed grader, a regex pattern changed to
    None, or a deleted case leaves stale Markdown behind that the runner still
    discovers and scores. Only directories containing a generated `prompt.md`
    are removed, so `variants/`, `results/`, and loose files like README.md
    survive.
    """
    if not ROOT.exists():
        return
    for child in sorted(ROOT.iterdir()):
        if not child.is_dir() or child.name in PRESERVE:
            continue
        if not (child / "prompt.md").exists():
            continue
        shutil.rmtree(child)


def main() -> None:
    clear_generated_cases()
    for slug, question, rubrics, pattern in CASES:
        case_dir = ROOT / slug
        write_prompt(case_dir, question)
        for name, rubric in rubrics:
            write_llm(case_dir, name, textwrap.dedent(rubric))
        if slug.startswith(RECALL_CRITICAL):
            write_llm(
                case_dir,
                "exhaustive-by-construction",
                EXHAUSTIVE_BY_CONSTRUCTION,
            )
        if pattern:
            write_regex(case_dir, "names-graph-command", pattern)
        # Written on every case, negatives included. On a should-NOT-fire case a
        # red indicator is EXPECTED and fine — it is how over-triggering becomes
        # visible when a description is broadened. It is display-only either way
        # and cannot move the score.
        write_skill_fired(case_dir)
        print(f"wrote {slug}")

    for tag, preamble, cases in EXTRA_FLOWS:
        for slug, question, rubrics, pattern in cases:
            case_dir = ROOT / slug
            write_prompt(case_dir, question, tag=tag, preamble=preamble)
            for name, rubric in rubrics:
                write_llm(case_dir, name, textwrap.dedent(rubric))
            if pattern:
                write_regex(case_dir, "names-graph-command", pattern)
            write_skill_fired(case_dir)
            print(f"wrote {slug}  [{tag}]")


if __name__ == "__main__":
    main()
