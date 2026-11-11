const assert = require("node:assert/strict");
const { spawn } = require("node:child_process");
const { once } = require("node:events");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { captureBuildOutput } = require("./native-build-output.cjs");
const {
  Capture,
} = require("../../test-cases/native-build-diagnostics/backpressure/slow-destination.cjs");

const root = path.resolve(__dirname, "../..");
const fixture = path.join(root, "test-cases/native-build-diagnostics/backpressure");
const writer = path.join(fixture, "prolific-writer.cjs");
const payload = require("../../test-cases/native-build-diagnostics/backpressure/payload.json");
const limit = 5000;

function expected(prefix) {
  return Buffer.concat(
    Array.from({ length: payload.chunks }, (_, index) => {
      const value = Buffer.alloc(payload.chunkBytes, prefix === payload.stdoutPrefix ? 0x53 : 0x45);
      Buffer.from(`${prefix}:${String(index).padStart(4, "0")}:`).copy(value);
      return value;
    }),
  );
}

const expectedStdout = expected(payload.stdoutPrefix);
const expectedStderr = expected(payload.stderrPrefix);

function waitFor(predicate, label) {
  const started = Date.now();
  return new Promise((resolve, reject) => {
    const poll = () => {
      if (predicate()) return resolve();
      if (Date.now() - started > limit) return reject(new Error(`timed out waiting for ${label}`));
      setTimeout(poll, 5);
    };
    poll();
  });
}

function childWriter() {
  return spawn(process.execPath, [writer], { stdio: ["ignore", "pipe", "pipe"] });
}

function assertMarkersInOrder(buffer, prefix) {
  let cursor = 0;
  for (let index = 0; index < payload.chunks; index++) {
    const marker = Buffer.from(`${prefix}:${String(index).padStart(4, "0")}:`);
    const found = buffer.indexOf(marker, cursor);
    assert.notEqual(found, -1, `${prefix} record ${index} must survive in cargo.log`);
    cursor = found + marker.length;
  }
}

function assertExactOutput(stdout, stderr, log) {
  assert.deepEqual(stdout.bytes, expectedStdout);
  assert.deepEqual(stderr.bytes, expectedStderr);
  assert.equal(log.bytes.length, expectedStdout.length + expectedStderr.length);
  assertMarkersInOrder(log.bytes, payload.stdoutPrefix);
  assertMarkersInOrder(log.bytes, payload.stderrPrefix);
}

async function settleChild(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  child.kill("SIGKILL");
  await Promise.race([once(child, "close"), new Promise((resolve) => setTimeout(resolve, 1000))]);
}

for (const slowName of ["stdout", "stderr"]) {
  test(
    `slow ${slowName} backpressure is bounded and leaves the other stream moving`,
    { timeout: 10000 },
    async () => {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-output-slow-"));
      const child = childWriter();
      const stdout = new Capture({ hold: slowName === "stdout" });
      const stderr = new Capture({ hold: slowName === "stderr" });
      const log = new Capture();
      const finish = captureBuildOutput(child, directory, { stdout, stderr, createLog: () => log });
      try {
        const slowStream = child[slowName];
        const fastSink = slowName === "stdout" ? stderr : stdout;
        const expectedFast = slowName === "stdout" ? expectedStderr : expectedStdout;
        const slowSink = slowName === "stdout" ? stdout : stderr;
        await waitFor(
          () => slowStream.isPaused() && fastSink.bytes.length === expectedFast.length,
          `${slowName} pressure and other stream completion`,
        );
        const bound = slowSink.writableHighWaterMark + (slowStream.readableHighWaterMark || 65536);
        assert.ok(
          slowSink.writableLength <= bound,
          `${slowName} buffer ${slowSink.writableLength} exceeds ${bound}`,
        );
        slowSink.release();
        const [code, signal] = await once(child, "close");
        assert.equal(code, 0);
        assert.equal(signal, null);
        finish();
        stdout.end();
        stderr.end();
        await Promise.all([once(stdout, "finish"), once(stderr, "finish"), once(log, "finish")]);
        assertExactOutput(stdout, stderr, log);
      } finally {
        await settleChild(child);
        fs.rmSync(directory, { recursive: true, force: true });
      }
    },
  );
}

test(
  "shared log backpressure pauses both sources and resumes exactly after drain",
  { timeout: 10000 },
  async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-output-log-drain-"));
    const child = childWriter();
    const stdout = new Capture();
    const stderr = new Capture();
    const log = new Capture({ hold: true });
    const finish = captureBuildOutput(child, directory, { stdout, stderr, createLog: () => log });
    try {
      await waitFor(
        () => child.stdout.isPaused() && child.stderr.isPaused(),
        "shared log backpressure",
      );
      const bound =
        log.writableHighWaterMark +
        2 *
          Math.max(
            child.stdout.readableHighWaterMark || 65536,
            child.stderr.readableHighWaterMark || 65536,
          );
      assert.ok(log.writableLength <= bound, `log buffer ${log.writableLength} exceeds ${bound}`);
      log.release();
      const [code, signal] = await once(child, "close");
      assert.equal(code, 0);
      assert.equal(signal, null);
      finish();
      stdout.end();
      stderr.end();
      await Promise.all([once(stdout, "finish"), once(stderr, "finish"), once(log, "finish")]);
      assertExactOutput(stdout, stderr, log);
    } finally {
      await settleChild(child);
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

test(
  "log failure detaches only the logger and drains both output streams",
  { timeout: 10000 },
  async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-output-log-error-"));
    const child = childWriter();
    const stdout = new Capture();
    const stderr = new Capture();
    const log = new Capture({ hold: true });
    const finish = captureBuildOutput(child, directory, { stdout, stderr, createLog: () => log });
    try {
      await waitFor(
        () => child.stdout.isPaused() && child.stderr.isPaused(),
        "shared log backpressure",
      );
      log.fail(Object.assign(new Error("fixture disk full"), { code: "ENOSPC" }));
      const [code, signal] = await once(child, "close");
      assert.equal(code, 0);
      assert.equal(signal, null);
      finish();
      stdout.end();
      stderr.end();
      await Promise.all([once(stdout, "finish"), once(stderr, "finish")]);
      assert.deepEqual(stdout.bytes, expectedStdout);
      assert.deepEqual(stderr.bytes, expectedStderr);
      assert.match(fs.readFileSync(path.join(directory, "log-error.txt"), "utf8"), /ENOSPC/);
    } finally {
      await settleChild(child);
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

test(
  "cancel destroys permanently stalled destinations and lets the child close",
  { timeout: 10000 },
  async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-output-cancel-"));
    const child = childWriter();
    const stdout = new Capture({ hold: true });
    const stderr = new Capture({ hold: true });
    const log = new Capture({ hold: true });
    const finish = captureBuildOutput(child, directory, { stdout, stderr, createLog: () => log });
    try {
      await waitFor(
        () => child.stdout.isPaused() && child.stderr.isPaused(),
        "stalled output destinations",
      );
      finish.cancel();
      child.kill("SIGTERM");
      const [code, signal] = await once(child, "close");
      if (process.platform === "win32") assert.ok(signal === "SIGTERM" || code !== 0);
      else assert.deepEqual([code, signal], [null, "SIGTERM"]);
      finish();
    } finally {
      await settleChild(child);
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
