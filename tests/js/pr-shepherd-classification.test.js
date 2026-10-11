const assert = require("node:assert/strict");
const notices = require("../../fixtures/pr-shepherd/bot-notices.json");
const mixedNotices = require("../../fixtures/pr-shepherd/mixed-notices.json");

const classifiers = [
  ["codex", () => import("../../.pr-shepherd/classification/codex-notices.mts")],
  ["coderabbit", () => import("../../.pr-shepherd/classification/coderabbit-notices.mts")],
];

for (const [family, loadRule] of classifiers) {
  test(`${family} classifiers suppress observed notices and preserve review feedback`, async () => {
    const { default: rule } = await loadRule();
    for (const notice of notices.filter((notice) => notice.name.startsWith(`${family}-`))) {
      const item = { ...notice, kind: "pr-comment", authorType: "Bot", id: notice.name };
      assert.equal(rule(item)?.suppress, true, notice.name);
      // Ignore classifications hide output without minimizing mutable status comments.
      assert.equal(rule(item)?.autoResolve, undefined);
      assert.equal(rule({ ...item, author: `${item.author.toUpperCase()}[bot]` })?.suppress, true);
      assert.equal(rule({ ...item, author: "another-bot" }), null);
      for (const kind of ["review-thread", "review-summary", "changes-requested"]) {
        assert.equal(rule({ ...item, kind }), null, `${notice.name}: ${kind}`);
      }
      assert.equal(
        rule({ ...item, body: "The SQL parser drops the second UNION branch. Add a regression." }),
        null,
      );
    }
    // Mutable bot comments can acquire findings after a notice was first seen.
    for (const notice of mixedNotices.filter((notice) => notice.name.startsWith(`${family}-`))) {
      assert.equal(
        rule({ ...notice, kind: "pr-comment", authorType: "Bot", id: notice.name }),
        null,
        notice.name,
      );
    }
  });
}

test("notice markers alone do not suppress substantive comments", async () => {
  const { default: codex } = await import("../../.pr-shepherd/classification/codex-notices.mts");
  const { default: coderabbit } =
    await import("../../.pr-shepherd/classification/coderabbit-notices.mts");
  const item = { kind: "pr-comment", authorType: "Bot", id: "substantive" };
  assert.equal(
    codex({
      ...item,
      author: "chatgpt-codex-connector",
      body: "<!-- codex-pull-request-review-summary -->\nParser findings follow.",
    }),
    null,
  );
  for (const marker of ["rate limited", "skip review", "review in progress"]) {
    assert.equal(
      coderabbit({
        ...item,
        author: "coderabbitai",
        body: `<!-- This is an auto-generated comment: ${marker} by coderabbit.ai -->\nParser findings follow.`,
      }),
      null,
    );
  }
});
