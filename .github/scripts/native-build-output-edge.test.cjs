const assert = require("node:assert/strict");
const { spawn } = require("node:child_process");
const { once } = require("node:events");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { captureBuildOutput } = require("./native-build-output.cjs");

const fixture = path.resolve(__dirname, "../../test-cases/native-build-diagnostics/backpressure");
const {
  Capture,
} = require("../../test-cases/native-build-diagnostics/backpressure/slow-destination.cjs");
const payload = require("../../test-cases/native-build-diagnostics/backpressure/payload.json");
const total = payload.chunks * payload.chunkBytes;

function assertComplete(buffer, prefix) {
  let cursor = 0;
  for (let index = 0; index < payload.chunks; index++) {
    const marker = Buffer.from(`${prefix}:${String(index).padStart(4, "0")}:`);
    const found = buffer.indexOf(marker, cursor);
    assert.notEqual(found, -1, `${prefix} record ${index} must survive`);
    cursor = found + marker.length;
  }
}

async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (!predicate()) {
    assert.ok(Date.now() < deadline, "output did not reach the expected state");
    await new Promise((resolve) => setTimeout(resolve, 5));
  }
}

function prepare({ stdoutHold = true, logHold = false } = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-output-edge-"));
  const child = spawn(process.execPath, [path.join(fixture, "prolific-writer.cjs")], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stdout = new Capture({ hold: stdoutHold });
  const stderr = new Capture();
  const log = new Capture({ hold: logHold });
  const finish = captureBuildOutput(child, directory, { stdout, stderr, createLog: () => log });
  return {
    directory,
    child,
    stdout,
    stderr,
    log,
    finish,
    close: () => once(child, "close", { signal: AbortSignal.timeout(5000) }),
    cleanup() {
      child.kill("SIGKILL");
      child.stdout.destroy();
      child.stderr.destroy();
      stdout.destroy();
      stderr.destroy();
      log.destroy();
      fs.rmSync(directory, { recursive: true, force: true });
    },
  };
}

async function finishHealthy(input, sinks) {
  const finished = sinks.map((sink) => once(sink, "finish", { signal: AbortSignal.timeout(5000) }));
  input.finish();
  for (const sink of sinks) if (sink !== input.log) sink.end();
  await Promise.all(finished);
}

for (const failure of ["error", "close"]) {
  test(
    `paused console ${failure} preserves the healthy console and complete log`,
    { timeout: 10000 },
    async () => {
      const input = prepare();
      try {
        await waitFor(() => input.child.stdout.isPaused() && input.stderr.bytes.length === total);
        const closed = input.close();
        if (failure === "error")
          input.stdout.fail(Object.assign(new Error("fixture console failed"), { code: "EPIPE" }));
        else input.stdout.destroy();
        assert.deepEqual(await closed, [0, null]);
        await finishHealthy(input, [input.stderr, input.log]);
        assert.equal(input.stderr.bytes.length, total);
        assertComplete(input.stderr.bytes, payload.stderrPrefix);
        assert.equal(input.log.bytes.length, 2 * total);
        assertComplete(input.log.bytes, payload.stdoutPrefix);
        assertComplete(input.log.bytes, payload.stderrPrefix);
      } finally {
        input.cleanup();
      }
    },
  );
}

test(
  "failed shared log does not resume stdout while its console remains blocked",
  { timeout: 10000 },
  async () => {
    const input = prepare({ logHold: true });
    try {
      await waitFor(() => input.child.stdout.isPaused() && input.child.stderr.isPaused());
      const blockedBytes = input.stdout.writableLength;
      input.log.fail(Object.assign(new Error("fixture disk full"), { code: "ENOSPC" }));
      await waitFor(() => input.stderr.bytes.length === total);
      assert.ok(input.child.stdout.isPaused());
      assert.equal(input.stdout.writableLength, blockedBytes);
      const closed = input.close();
      input.stdout.release();
      assert.deepEqual(await closed, [0, null]);
      await finishHealthy(input, [input.stdout, input.stderr]);
      assert.equal(input.stdout.bytes.length, total);
      assert.equal(input.stderr.bytes.length, total);
      assertComplete(input.stdout.bytes, payload.stdoutPrefix);
      assertComplete(input.stderr.bytes, payload.stderrPrefix);
      assert.match(fs.readFileSync(path.join(input.directory, "log-error.txt"), "utf8"), /ENOSPC/);
    } finally {
      input.cleanup();
    }
  },
);

test(
  "cancellation before the first chunk releases destinations that stall later",
  { timeout: 10000 },
  async () => {
    const input = prepare({ logHold: true });
    try {
      assert.equal(input.stdout.writableLength, 0);
      assert.equal(input.log.writableLength, 0);
      const closed = input.close();
      input.finish.cancel();
      assert.deepEqual(await closed, [0, null]);
      input.finish();
      assert.ok(input.stdout.destroyed);
      assert.ok(input.log.destroyed);
    } finally {
      input.cleanup();
    }
  },
);

test(
  "cancellation flushes healthy asynchronous destinations before abandoning stalled output",
  { timeout: 10000 },
  async () => {
    const input = prepare({ stdoutHold: false });
    function expected(prefix) {
      return Buffer.concat(
        Array.from({ length: payload.chunks }, (_, index) => {
          const chunk = Buffer.alloc(
            payload.chunkBytes,
            prefix === payload.stdoutPrefix ? 0x53 : 0x45,
          );
          Buffer.from(`${prefix}:${String(index).padStart(4, "0")}:`).copy(chunk);
          return chunk;
        }),
      );
    }
    try {
      const closed = input.close();
      input.finish.cancel();
      assert.deepEqual(await closed, [0, null]);
      await finishHealthy(input, [input.stdout, input.stderr, input.log]);
      assert.deepEqual(input.stdout.bytes, expected(payload.stdoutPrefix));
      assert.deepEqual(input.stderr.bytes, expected(payload.stderrPrefix));
      assert.equal(input.log.bytes.length, 2 * total);
      assertComplete(input.log.bytes, payload.stdoutPrefix);
      assertComplete(input.log.bytes, payload.stderrPrefix);
      assert.ok(input.stdout.writableFinished);
      assert.ok(input.stderr.writableFinished);
      assert.ok(input.log.writableFinished);
    } finally {
      input.cleanup();
    }
  },
);
