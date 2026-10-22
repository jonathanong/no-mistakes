const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const repoRoot = join(__dirname, "..", "..");
const selector = join(repoRoot, ".github", "scripts", "timeout-policy-browser-scope.cjs");
const { timeoutPolicyBrowserChanged } = require(selector);
const modified = (paths) => paths.map((path) => ({ status: "M", path }));

const BROWSER_TESTS = [
  "packages/no-mistakes/test-timeout-policy/browser-route.test.mjs",
  // These also spawn Vitest Browser Mode; leaving them in test:ci would still need Chromium.
  "packages/no-mistakes/test-timeout-policy/storybook-route.test.mjs",
  "packages/no-mistakes/test-timeout-policy/review-context-options.test.mjs",
];
const NODE_POLICY_CONTRACTS =
  "packages/no-mistakes/test-timeout-policy/browser-policy-contracts.test.mjs";

function pkg(vitest = "^5.0.4", playwright = "1.64.0") {
  return JSON.stringify({
    devDependencies: {
      vitest,
      "@vitest/browser-playwright": "5.0.4",
      playwright,
    },
  });
}

test("runs when timeout-policy sources, entrypoints, or the workflow change", () => {
  for (const path of [
    "packages/no-mistakes/test-timeout-policy/browser-route.test.mjs",
    "packages/no-mistakes/test-timeout-policy/plugin.mjs",
    "fixtures/test-timeout-policy/browser-timeout-child.mjs",
    "packages/no-mistakes/package.json",
    ".github/workflows/ci.yml",
    ".github/scripts/timeout-policy-browser-scope.cjs",
    "tests/js/timeout-policy-browser-scope.test.js",
    "vitest.config.js",
  ]) {
    assert.equal(timeoutPolicyBrowserChanged(modified([path])), true, path);
  }
});

test("ignores unrelated paths and untracked dependency bumps", () => {
  assert.equal(timeoutPolicyBrowserChanged(modified(["docs/architecture.md"])), false);
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: { base: pkg(), head: pkg() },
    }),
    false,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: { base: pkg("^5.0.4"), head: pkg("^5.0.5") },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["pnpm-lock.yaml"]), {
      lockfile: {
        base: "  playwright@1.64.0:\n",
        head: "  playwright@1.64.0:\n",
      },
    }),
    false,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["pnpm-lock.yaml"]), {
      lockfile: {
        base: "  playwright@1.64.0:\n",
        head: "  playwright@1.65.0:\n",
      },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: { base: pkg(), head: null },
    }),
    true,
  );
});

test("CLI reads NUL-delimited statuses and supports an explicit full fallback", () => {
  const run = (input, args = []) =>
    spawnSync(process.execPath, [selector, ...args], { input, encoding: "utf8" });
  assert.equal(run("", ["--all"]).stdout, "changed=true\n");
  const docs = run("M\0docs/architecture.md\0");
  assert.equal(docs.status, 0);
  assert.equal(docs.stdout, "changed=false\n");
  const policy = run("M\0packages/no-mistakes/test-timeout-policy/plugin.mjs\0");
  assert.equal(policy.status, 0);
  assert.equal(policy.stdout, "changed=true\n");
  assert.notEqual(run("M\0docs/architecture.md").status, 0);
});

test("test:ci excludes Chromium browser-mode files that release still runs", () => {
  const pkgJson = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
  for (const file of BROWSER_TESTS) {
    assert.match(
      pkgJson.scripts["test:ci"],
      new RegExp(`--exclude ${file.replaceAll(".", "\\.")}`),
    );
    assert.doesNotMatch(pkgJson.scripts.test, /--exclude/);
    assert.match(
      pkgJson.scripts["test:timeout-policy-browser"],
      new RegExp(file.replaceAll(".", "\\.")),
    );
  }
  // Node plugin contracts cover plugin.mjs; excluding them drops the 98% branch floor.
  assert.doesNotMatch(
    pkgJson.scripts["test:ci"],
    new RegExp(`--exclude ${NODE_POLICY_CONTRACTS.replaceAll(".", "\\.")}`),
  );
  assert.doesNotMatch(
    pkgJson.scripts["test:timeout-policy-browser"],
    new RegExp(NODE_POLICY_CONTRACTS.replaceAll(".", "\\.")),
  );
});

test("CI installs Chromium only on the path-filtered browser job", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const jsJob = workflow.match(/^ {2}javascript-tests:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  const browserJob = workflow.match(
    /^ {2}timeout-policy-browser-tests:[\s\S]*?(?=^ {2}[a-z])/m,
  )?.[0];
  const gate = workflow.match(/^ {2}tests-gate:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  assert.ok(jsJob && browserJob && gate);
  assert.doesNotMatch(jsJob, /playwright install/);
  assert.match(browserJob, /playwright install --with-deps chromium/);
  assert.match(browserJob, /needs: changes/);
  assert.match(browserJob, /needs\.changes\.outputs\.timeout-policy-browser == 'true'/);
  assert.match(gate, /timeout-policy-browser-tests/);
  const script = gate
    .split("run: |\n")[1]
    ?.trim()
    .replace(/^ {10}/gm, "");
  assert.ok(script);
  const run = (browser) =>
    spawnSync("bash", ["-e", "-c", script], {
      env: {
        ...process.env,
        QUALITY_RESULT: "success",
        JAVASCRIPT_RESULT: "success",
        NATIVE_RESULT: "success",
        RUST_RESULT: "success",
        STANDALONE_RESULT: "success",
        CHANGE_RESULT: "success",
        BROWSER_CHANGED: browser.changed,
        BROWSER_RESULT: browser.result,
      },
    });
  assert.equal(run({ changed: "false", result: "skipped" }).status, 0);
  assert.equal(run({ changed: "true", result: "success" }).status, 0);
  assert.notEqual(run({ changed: "true", result: "failure" }).status, 0);
  assert.notEqual(run({ changed: "false", result: "failure" }).status, 0);
});

test("release validation still installs Chromium and runs the full suite", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const js = workflow.match(/^ {2}validate-js:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  assert.ok(js);
  assert.match(js, /playwright install --with-deps chromium/);
  assert.match(js, /run: pnpm test\n/);
  assert.doesNotMatch(js, /test:ci|test:timeout-policy-browser/);
});
