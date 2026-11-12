const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const repoRoot = join(__dirname, "../..");
const fixture = JSON.parse(
  readFileSync(
    join(repoRoot, "test-cases/rust-validation/fixture/lychee-codspeed-challenge.json"),
    "utf8",
  ),
);

test("Lychee ignores only the CodSpeed profile blocked by Vercel", () => {
  const ignoredPatterns = readFileSync(join(repoRoot, ".lycheeignore"), "utf8")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"));
  const matchingPatterns = ignoredPatterns.filter((pattern) =>
    new RegExp(pattern).test(fixture.exactChallengeUrl),
  );

  assert.equal(matchingPatterns.length, 1);
  assert.equal(matchingPatterns[0], "^https://app[.]codspeed[.]io/jonathanong/no-mistakes[?]utm_source=badge$");
  for (const url of fixture.mustRemainChecked) {
    assert.equal(
      ignoredPatterns.some((pattern) => new RegExp(pattern).test(url)),
      false,
      `${url} must remain under link checking`,
    );
  }

  const workflow = readFileSync(join(repoRoot, ".github/workflows/ci.yml"), "utf8");
  assert.doesNotMatch(workflow, /--accept[^\n]*(?:429|4xx)/);
});
