const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { mkdtempSync, mkdirSync, writeFileSync, chmodSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const script = path.join(__dirname, "install-wheels.sh");

function invocations(logText) {
  return logText
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => line.split("\0").filter((part) => part.length > 0));
}

function run(t, { existing = null, failCommand = "", writeWheel = "1" } = {}) {
  const root = mkdtempSync(path.join(tmpdir(), "accounting-wheels-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const bin = path.join(root, "bin");
  const wheels = path.join(root, "wheels");
  const venv = path.join(root, "venv");
  const requirements = path.join(root, "requirements.txt");
  const log = path.join(root, "pip.log");
  mkdirSync(bin);
  mkdirSync(wheels);
  writeFileSync(requirements, "msgpack==1.2.3\n");
  if (existing) writeFileSync(path.join(wheels, existing), "");
  writeFileSync(
    path.join(bin, "pip-impl"),
    `#!/bin/sh
printf '%s\\0' "$@" >> "$LOG"
printf '\\n' >> "$LOG"
if [ -n "$FAIL_COMMAND" ] && [ "$1" = "$FAIL_COMMAND" ]; then
  exit 1
fi
if [ "$1" = "download" ] && [ "$WRITE_WHEEL" != "0" ]; then
  : > "$WHEELS/msgpack-1.2.3-py3-none-any.whl"
fi
exit 0
`,
  );
  chmodSync(path.join(bin, "pip-impl"), 0o755);
  writeFileSync(
    path.join(bin, "python3"),
    `#!/bin/sh
if [ "$1" = "-m" ] && [ "$2" = "venv" ]; then
  mkdir -p "$3/bin"
  cp "$PIP_IMPL" "$3/bin/pip"
  chmod +x "$3/bin/pip"
  exit 0
fi
echo "unexpected python3: $*" >&2
exit 1
`,
  );
  chmodSync(path.join(bin, "python3"), 0o755);
  const result = spawnSync("bash", [script, requirements, wheels, venv], {
    encoding: "utf8",
    env: {
      PATH: `${bin}:/usr/bin:/bin`,
      LOG: log,
      WHEELS: wheels,
      PIP_IMPL: path.join(bin, "pip-impl"),
      FAIL_COMMAND: failCommand,
      WRITE_WHEEL: writeWheel,
    },
  });
  const logText = readFileSync(log, "utf8");
  return { result, calls: invocations(logText), requirements, wheels };
}

test("an empty wheel directory downloads with two retries, then installs offline", (t) => {
  const { result, calls, requirements, wheels } = run(t);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(calls.length, 2);
  assert.deepEqual(calls[0], [
    "download",
    "--disable-pip-version-check",
    "--only-binary=:all:",
    "--no-deps",
    "--retries",
    "2",
    "--timeout",
    "20",
    "--dest",
    wheels,
    "--requirement",
    requirements,
  ]);
  assert.deepEqual(calls[1], [
    "install",
    "--disable-pip-version-check",
    "--only-binary=:all:",
    "--no-deps",
    "--no-index",
    "--find-links",
    wheels,
    "--retries",
    "0",
    "--timeout",
    "20",
    "--requirement",
    requirements,
  ]);
});

test("a restored wheel skips the download", (t) => {
  const { result, calls } = run(t, { existing: "msgpack-1.2.3-py3-none-any.whl" });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], "install");
  assert.ok(calls[0].includes("--no-index"));
});

test("a non-wheel file does not count as a restored wheel", (t) => {
  const { result, calls } = run(t, { existing: "notes.txt" });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(calls[0][0], "download");
});

test("a failed download does not install", (t) => {
  const { result, calls } = run(t, { failCommand: "download" });
  assert.notEqual(result.status, 0);
  assert.deepEqual(
    calls.map((call) => call[0]),
    ["download"],
  );
});

test("a download that writes no wheel does not install", (t) => {
  const { result, calls } = run(t, { writeWheel: "0" });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /pinned wheels were not downloaded/);
  assert.deepEqual(
    calls.map((call) => call[0]),
    ["download"],
  );
});

test("a failed offline install fails the script", (t) => {
  const { result, calls } = run(t, { failCommand: "install" });
  assert.notEqual(result.status, 0);
  assert.deepEqual(
    calls.map((call) => call[0]),
    ["download", "install"],
  );
});

test("missing arguments are rejected before pip runs", () => {
  const result = spawnSync("bash", [script], { encoding: "utf8" });
  assert.equal(result.status, 2);
  assert.match(result.stderr, /usage: install-wheels.sh/);
});
