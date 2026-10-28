const { assert, test, readFileSync, join, packageRoot } = require("./context");
const { decorateInvocationError } = require(join(packageRoot, "invocation-errors.js"));

const MARKER = "\nno-mistakes-error-json: ";

test("lock failures expose a code and structured details without the marker text", () => {
  const details = {
    kind: "lock_busy",
    phase: "invocation.lock",
    message: "another no-mistakes invocation is already running (held by pid 4242)",
    exitCode: 75,
    holder: { pid: 4242, cwd: "/work/other-repo", command: "no-mistakes check" },
  };
  const error = new Error(`${details.message}${MARKER}${JSON.stringify(details)}`);
  assert.equal(decorateInvocationError(error), error);
  assert.equal(error.message, details.message);
  assert.equal(error.code, "NO_MISTAKES_LOCK_BUSY");
  assert.deepEqual(error.details, details);
});

test("deadline and lock timeout kinds map to distinct codes", () => {
  for (const [kind, code] of [
    ["lock_timeout", "NO_MISTAKES_LOCK_TIMEOUT"],
    ["deadline", "NO_MISTAKES_DEADLINE"],
  ]) {
    const error = decorateInvocationError(new Error(`x${MARKER}${JSON.stringify({ kind })}`));
    assert.equal(error.code, code);
  }
});

test("errors without a report, or with an unreadable one, are left untouched", () => {
  const plain = new Error("boom");
  assert.equal(decorateInvocationError(plain).code, undefined);
  const garbled = new Error(`boom${MARKER}{not json`);
  decorateInvocationError(garbled);
  assert.equal(garbled.code, undefined);
  assert.ok(garbled.message.includes("not json"));
  assert.equal(decorateInvocationError("not an error"), "not an error");
});

test("lockStatus and structured invocation errors have public declarations", () => {
  const types = readFileSync(join(packageRoot, "invocation-types.d.ts"), "utf8");
  assert.match(types, /export interface LockStatus \{/);
  assert.match(types, /state: "held" \| "free";/);
  assert.match(types, /export interface LockHolder \{/);
  assert.match(
    types,
    /export type InvocationErrorKind = "lock_busy" \| "lock_timeout" \| "deadline";/,
  );
  assert.match(types, /export interface InvocationErrorDetails \{/);
  assert.match(types, /export interface InvocationError extends Error \{/);
  assert.match(
    readFileSync(join(packageRoot, "index-ci-infra.d.ts"), "utf8"),
    /export function lockStatus\(\): Promise<import\("\.\/invocation-types"\)\.LockStatus>;/,
  );
});
