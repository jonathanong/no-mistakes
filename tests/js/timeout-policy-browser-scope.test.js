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

const DEFAULT_SCRIPTS = {
  "test:ci": "vitest run --coverage",
  "test:timeout-policy-browser": "vitest run browser",
  "fmt:js": "oxfmt",
};

function pkg(vitest = "^5.0.4", playwright = "1.64.0", storybookReact = "10.6.1", extra = {}) {
  return JSON.stringify({
    scripts: extra.scripts ?? DEFAULT_SCRIPTS,
    devDependencies: {
      vitest,
      "@vitest/browser-playwright": "5.0.4",
      playwright,
      "@storybook/react": storybookReact,
      "@storybook/react-vite": "10.6.1",
      react: extra.react ?? "19.3.0",
      "react-dom": extra.reactDom ?? "19.3.0",
      vite: extra.vite ?? "7.1.0",
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
    "pnpm-workspace.yaml",
    "tests/js/setup.js",
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
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: { base: pkg(), head: pkg("^5.0.4", "1.64.0", "10.6.2") },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["pnpm-lock.yaml"]), {
      lockfile: {
        base: "  '@storybook/react-vite@10.6.1':\n",
        head: "  '@storybook/react-vite@10.6.2':\n",
      },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: {
        base: pkg(),
        head: pkg("^5.0.4", "1.64.0", "10.6.1", { react: "19.4.0" }),
      },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: {
        base: pkg(),
        head: pkg("^5.0.4", "1.64.0", "10.6.1", { reactDom: "19.4.0" }),
      },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: {
        base: pkg(),
        head: pkg("^5.0.4", "1.64.0", "10.6.1", { vite: "7.2.0" }),
      },
    }),
    true,
  );
});

test("runs when root package.json test scripts change", () => {
  const withScripts = (scripts) =>
    pkg("^5.0.4", "1.64.0", "10.6.1", { scripts: { ...DEFAULT_SCRIPTS, ...scripts } });
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: {
        base: pkg(),
        head: withScripts({ "test:ci": "vitest run --coverage --exclude broken.mjs" }),
      },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: {
        base: pkg(),
        head: withScripts({
          "test:timeout-policy-browser": "vitest run --coverage=false missing.mjs",
        }),
      },
    }),
    true,
  );
  assert.equal(
    timeoutPolicyBrowserChanged(modified(["package.json"]), {
      packageJson: {
        base: pkg(),
        head: withScripts({ "fmt:js": "oxfmt --write ." }),
      },
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

test("CI installs Chromium only on browser jobs and runs them on every PR", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const browserJob = workflow.match(
    /^ {2}timeout-policy-browser-tests:[\s\S]*?(?=^ {2}[a-z])/m,
  )?.[0];
  const jsJob = workflow.match(/^ {2}javascript-shards:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  const gate = workflow.match(/^ {2}tests-gate:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  assert.ok(browserJob && jsJob && gate);
  assert.doesNotMatch(jsJob, /playwright install/);
  assert.match(browserJob, /playwright install chromium/);
  assert.doesNotMatch(browserJob, /playwright install --with-deps/);
  assert.doesNotMatch(browserJob, /needs\.changes\.outputs\.timeout-policy-browser == 'true'/);
  assert.match(gate, /timeout-policy-browser-tests/);
});

test("browser qualification retains lifecycle artifacts after failures within existing deadlines", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const job = workflow.match(/^ {2}timeout-policy-browser-tests:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  assert.ok(job);
  assert.match(job, /timeout-minutes: 20/);
  assert.match(
    job,
    /SDK_CHILD_RECEIPT_FILE: \$\{\{ runner.temp \}\}\/timeout-policy-browser\/receipts.jsonl/,
  );
  assert.match(
    job,
    /SDK_CHILD_HEAD_SHA: \$\{\{ github.event.pull_request.head.sha \|\| github.sha \}\}/,
  );
  assert.match(job, /mkdir -p "\$RUNNER_TEMP\/timeout-policy-browser"/);
  assert.match(job, /pnpm run test:timeout-policy-browser/);
  const upload = job.split("- name: Upload timeout-policy browser lifecycle\n")[1];
  assert.ok(upload);
  assert.match(upload, /if: \$\{\{ always\(\) \}\}/);
  assert.match(upload, /timeout-minutes: 2/);
  assert.match(upload, /uses: actions\/upload-artifact@[a-f0-9]{40} # v7/);
  assert.match(upload, /path: \$\{\{ runner.temp \}\}\/timeout-policy-browser/);
  assert.match(upload, /retention-days: 7/);
});

test("browser qualification compares the original Node runtime without replacing the current lane", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const job = workflow.match(/^ {2}timeout-policy-browser-tests:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  assert.ok(job);
  assert.match(job, /name: \$\{\{ matrix.name \}\}/);
  assert.match(job, /fail-fast: false/);
  assert.match(job, /- name: Timeout-policy browser tests\n {12}runtime: node@24\n/);
  assert.match(job, /runtime: node@24\.22\.2\n/);
  assert.match(job, /runtime: \$\{\{ matrix.runtime \}\}/);
  assert.match(job, /artifact: timeout-policy-browser-lifecycle\n/);
  assert.match(job, /artifact: timeout-policy-browser-lifecycle-node-24\.22\.2\n/);
  assert.match(job, /name: \$\{\{ matrix.artifact \}\}/);
});

test("release validation gives browser and other JavaScript tests separate timeouts", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const js = workflow.match(/^ {2}validate-js:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  assert.ok(js);
  assert.match(js, /playwright install chromium/);
  assert.doesNotMatch(js, /playwright install --with-deps/);
  assert.match(
    js,
    /- name: Run JavaScript tests\n {8}timeout-minutes: 5\n {8}run: pnpm run test:ci\n/,
  );
  assert.match(
    js,
    /- name: Run timeout-policy browser tests\n {8}timeout-minutes: 5\n {8}run: pnpm run --if-present test:timeout-policy-browser\n/,
  );
  assert.doesNotMatch(js, /run: pnpm test\n/);
});

for (const [manifest, exitCode, runsBrowser] of [
  ["current", 0, true],
  ["older", 0, false],
  ["current", 7, true],
]) {
  test(`release browser step supports ${manifest} manifest with script exit ${exitCode}`, () => {
    const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
    const command = workflow.match(
      /- name: Run timeout-policy browser tests\n {8}timeout-minutes: 5\n {8}run: ([^\n]+)/,
    )?.[1];
    assert.ok(command);
    const result = spawnSync("bash", ["-e", "-c", command], {
      cwd: join(repoRoot, "test-cases/release-validation/browser-script/fixture", manifest),
      env: { ...process.env, BROWSER_FIXTURE_EXIT_CODE: String(exitCode) },
      encoding: "utf8",
      timeout: 15_000,
    });
    assert.ifError(result.error);
    assert.equal(result.status === 0, exitCode === 0, result.stdout + result.stderr);
    assert.equal(result.stdout.includes("browser-suite-executed"), runsBrowser);
  });
}
