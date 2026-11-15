const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");

const workflow = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");
const job = workflowJob(workflow, "quality-source");

function step(name) {
  const body = job.split(`      - name: ${name}\n`)[1]?.split(/\n      - /)[0];
  assert.ok(body, `quality-source must retain ${name}`);
  return body;
}

test("frontend memory accounting restores a requirements-keyed wheelhouse and installs offline", () => {
  const version = step("Read Python version for frontend memory accounting");
  const restore = step("Restore frontend memory accounting wheels");
  const run = step("Test frontend memory accounting");
  const save = step("Save frontend memory accounting wheels");
  assert.ok(
    job.indexOf("- name: Read Python version for frontend memory accounting") <
      job.indexOf("- name: Restore frontend memory accounting wheels") &&
      job.indexOf("- name: Restore frontend memory accounting wheels") <
        job.indexOf("- name: Test frontend memory accounting") &&
      job.indexOf("- name: Test frontend memory accounting") <
        job.indexOf("- name: Save frontend memory accounting wheels"),
  );

  assert.match(version, /\n        timeout-minutes: 1\n/);
  assert.match(
    version,
    /hashFiles\('\.github\/scripts\/frontend-memory-accounting\/requirements\.txt'\)/,
  );
  assert.match(
    version,
    /cache-primary-key=frontend-memory-accounting-wheels-v1-\$\{RUNNER_OS\}-\$\{RUNNER_ARCH\}-py\$\{version\}-\$\{requirements_hash\}/,
  );

  const restorePin = restore.match(/uses: actions\/cache\/restore@([a-f0-9]{40}) # v6\.1\.0/);
  const savePin = save.match(/uses: actions\/cache\/save@([a-f0-9]{40}) # v6\.1\.0/);
  assert.ok(restorePin && savePin);
  assert.equal(restorePin[1], savePin[1]);
  const samePin = workflow.match(
    new RegExp(`uses: actions\\/cache\\/restore@${restorePin[1]} # v6\\.1\\.0`, "g"),
  );
  assert.ok(samePin && samePin.length >= 2, "wheel restore reuses the workflow cache pin");
  for (const body of [restore, save]) {
    assert.match(body, /\n        timeout-minutes: 2\n/);
    assert.match(body, /key: \$\{\{ steps\.accounting-python\.outputs\.cache-primary-key \}\}/);
    assert.match(body, /path: \$\{\{ runner\.temp \}\}\/frontend-memory-accounting-wheels/);
  }
  assert.match(restore, /\n        continue-on-error: true\n/);
  assert.match(
    save,
    /if: \$\{\{ success\(\) && github\.ref == 'refs\/heads\/main' && steps\.accounting-wheels\.outputs\.cache-hit != 'true' \}\}/,
  );
  assert.match(save, /\n        continue-on-error: true\n/);
  assert.doesNotMatch(workflow, /uses: actions\/cache@[a-f0-9]{40}/);

  assert.match(run, /^        timeout-minutes: 3\n/);
  assert.match(run, /install-wheels\.test\.cjs/);
  assert.match(run, /frontend-memory-accounting-workflow\.test\.cjs/);
  assert.match(
    run,
    /bash \.github\/scripts\/frontend-memory-accounting\/install-wheels\.sh \\\n\s+\.github\/scripts\/frontend-memory-accounting\/requirements\.txt \\\n\s+"\$RUNNER_TEMP\/frontend-memory-accounting-wheels" \\\n\s+"\$RUNNER_TEMP\/frontend-memory-accounting-venv"/,
  );
  assert.match(run, /test_accounting\.py/);
  assert.doesNotMatch(run, /\bpip\b/);
  assert.doesNotMatch(run, /^\s+if:/m);
  assert.doesNotMatch(run, /continue-on-error/);
  assert.doesNotMatch(job, /--retries 0/);
});
