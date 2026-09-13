const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { Buffer } = require("node:buffer");
const { spawnSync } = require("node:child_process");
const test = globalThis.test || require("node:test").test;

const {
  encodeName,
  main,
  parseArgs,
  readWithSignal,
  reportCliFailure,
  tarballUrl,
  runIfMain,
  startFromCli,
  waitForNpmTarball,
} = require("./wait-npm-tarball");

function jsonResponse(status, body) {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => body,
    arrayBuffer: async () => Buffer.from(JSON.stringify(body)),
  };
}

function binaryResponse(status, body) {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => {
      throw new Error("tarball is not JSON");
    },
    arrayBuffer: async () => body,
  };
}

function packument(version, shasum, tarball) {
  return {
    versions: {
      [version]: {
        dist: { shasum, ...(tarball ? { tarball } : {}) },
      },
    },
  };
}

test("parses CLI flags and rejects incomplete argv", () => {
  assert.deepEqual(parseArgs(["--package", "pkg", "--version", "1.0.0"]), {
    package: "pkg",
    version: "1.0.0",
  });
  assert.throws(() => parseArgs(["pkg"]), /unexpected argument pkg/);
  assert.throws(() => parseArgs(["--package"]), /--package requires a value/);
  assert.throws(() => parseArgs(["--package", "--version"]), /--package requires a value/);
});

test("encodes scoped names and builds the default tarball URL", () => {
  assert.equal(encodeName("no-mistakes-darwin-arm64"), "no-mistakes-darwin-arm64");
  assert.equal(encodeName("@scope/pkg"), "@scope%2Fpkg");
  assert.equal(
    tarballUrl("https://registry.npmjs.org/", "no-mistakes-darwin-arm64", "0.61.6"),
    "https://registry.npmjs.org/no-mistakes-darwin-arm64/-/no-mistakes-darwin-arm64-0.61.6.tgz",
  );
});

test("returns the tarball once packument metadata and bytes match", async () => {
  const body = Buffer.from("native-package");
  const shasum = createHash("sha1").update(body).digest("hex");
  const result = await waitForNpmTarball({
    name: "no-mistakes-darwin-arm64",
    version: "0.61.6",
    timeoutMs: 0,
    fetchImpl: async (url) => {
      if (url.endsWith("/no-mistakes-darwin-arm64")) {
        return jsonResponse(
          200,
          packument(
            "0.61.6",
            shasum,
            "https://registry.npmjs.org/no-mistakes-darwin-arm64/-/no-mistakes-darwin-arm64-0.61.6.tgz",
          ),
        );
      }
      return binaryResponse(200, body);
    },
  });
  assert.equal(result.shasum, shasum);
  assert.equal(result.bytes, body.length);
});

test("retries metadata-only publishes until the tarball becomes fetchable", async () => {
  const body = Buffer.from("later-blob");
  const shasum = createHash("sha1").update(body).digest("hex");
  const sleeps = [];
  let attempts = 0;
  const result = await waitForNpmTarball({
    name: "pkg",
    version: "1.2.3",
    timeoutMs: 30_000,
    intervalMs: 5,
    now: (() => {
      let tick = 0;
      return () => {
        tick += 1;
        return tick * 10;
      };
    })(),
    sleep: async (ms) => {
      sleeps.push(ms);
    },
    fetchImpl: async (url) => {
      if (url.endsWith("/pkg")) {
        return jsonResponse(200, packument("1.2.3", shasum));
      }
      attempts += 1;
      if (attempts < 3) return binaryResponse(404, Buffer.from("missing"));
      return binaryResponse(200, body);
    },
  });
  assert.equal(attempts, 3);
  assert.deepEqual(sleeps, [5, 5]);
  assert.equal(result.shasum, shasum);
  assert.match(result.url, /pkg-1.2.3.tgz$/);
});

test("fails closed on missing fields, HTTP errors, and checksum mismatch", async () => {
  await assert.rejects(waitForNpmTarball({}), /name and version are required/);
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 0,
      fetchImpl: async () => jsonResponse(404, {}),
    }),
    /packument HTTP 404/,
  );
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 0,
      fetchImpl: async () => jsonResponse(200, { versions: {} }),
    }),
    /missing from the packument/,
  );
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 0,
      fetchImpl: async () => jsonResponse(200, { versions: { "1.0.0": { dist: {} } } }),
    }),
    /no dist.shasum/,
  );
  const body = Buffer.from("actual");
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 0,
      fetchImpl: async (url) => {
        if (url.endsWith("/pkg")) return jsonResponse(200, packument("1.0.0", "deadbeef"));
        return binaryResponse(200, body);
      },
    }),
    /tarball shasum/,
  );
});

test("timeout-ms 0 does not retry a missing tarball", async () => {
  let sleeps = 0;
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 0,
      sleep: async () => {
        sleeps += 1;
      },
      fetchImpl: async (url) => {
        if (url.endsWith("/pkg")) {
          return jsonResponse(200, packument("1.0.0", "abc"));
        }
        return binaryResponse(404, Buffer.from(""));
      },
    }),
    /tarball HTTP 404/,
  );
  assert.equal(sleeps, 0);
});

test("uses the default sleeper between retries", async () => {
  const body = Buffer.from("after-sleep");
  const shasum = createHash("sha1").update(body).digest("hex");
  let attempts = 0;
  const result = await waitForNpmTarball({
    name: "pkg",
    version: "1.0.0",
    timeoutMs: 1000,
    intervalMs: 0,
    fetchImpl: async (url) => {
      if (url.endsWith("/pkg")) return jsonResponse(200, packument("1.0.0", shasum));
      attempts += 1;
      if (attempts === 1) return binaryResponse(404, Buffer.from(""));
      return binaryResponse(200, body);
    },
  });
  assert.equal(attempts, 2);
  assert.equal(result.shasum, shasum);
});

test("wraps non-Error fetch failures and rejects invalid numeric flags", async () => {
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 0,
      fetchImpl: async () => {
        throw "cdn down";
      },
    }),
    /cdn down/,
  );
  await assert.rejects(
    main(["--package", "pkg", "--version", "1.0.0", "--timeout-ms", "nope"]),
    /--timeout-ms must be a number/,
  );
});

test("main writes JSON when the tarball is ready", async () => {
  const body = Buffer.from("cli");
  const shasum = createHash("sha1").update(body).digest("hex");
  const chunks = [];
  const writes = [];
  const originalFetch = globalThis.fetch;
  const originalWrite = process.stdout.write.bind(process.stdout);
  globalThis.fetch = async (url) => {
    if (String(url).endsWith("/pkg"))
      return jsonResponse(200, packument("2.0.0", shasum, `${url}-file.tgz`));
    return binaryResponse(200, body);
  };
  try {
    await main(
      ["--package", "pkg", "--version", "2.0.0", "--timeout-ms", "0", "--interval-ms", "1"],
      {
        stdout: { write: (chunk) => chunks.push(chunk) },
      },
    );
    process.stdout.write = (chunk) => {
      writes.push(String(chunk));
      return true;
    };
    await main(["--package", "pkg", "--version", "2.0.0", "--timeout-ms", "0"]);
  } finally {
    globalThis.fetch = originalFetch;
    process.stdout.write = originalWrite;
  }
  assert.equal(JSON.parse(chunks.join("")).shasum, shasum);
  assert.equal(JSON.parse(writes.join("")).shasum, shasum);
});

test("CLI reports usage errors without publishing", () => {
  const result = spawnSync(
    process.execPath,
    [require.resolve("./wait-npm-tarball.js"), "--package"],
    {
      encoding: "utf8",
    },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /--package requires a value/);
});

test("reportCliFailure writes the error and sets a nonzero exit", () => {
  const chunks = [];
  const io = { stderr: { write: (chunk) => chunks.push(chunk) }, exitCode: 0 };
  reportCliFailure(new Error("boom"), io);
  assert.equal(io.exitCode, 1);
  assert.deepEqual(chunks, ["boom\n"]);
});

test("aborts a stalled packument fetch at the deadline", async () => {
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 40,
      intervalMs: 0,
      fetchImpl: (_url, init = {}) =>
        new Promise((_, reject) => {
          assert.ok(init.signal, "packument fetch must receive AbortSignal");
          init.signal.addEventListener("abort", () => reject(new Error("aborted")), { once: true });
        }),
    }),
    /Timed out waiting for pkg@1.0.0 tarball/,
  );
});

test("aborts a stalled tarball body read at the deadline", async () => {
  const body = Buffer.from("unused");
  const shasum = createHash("sha1").update(body).digest("hex");
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 40,
      intervalMs: 0,
      fetchImpl: async (url) => {
        if (String(url).endsWith("/pkg")) return jsonResponse(200, packument("1.0.0", shasum));
        return {
          ok: true,
          status: 200,
          json: async () => ({}),
          arrayBuffer: () => new Promise(() => {}),
        };
      },
    }),
    /Timed out waiting for pkg@1.0.0 tarball/,
  );
});

test("stops when the overall deadline elapses before another fetch", async () => {
  let ticks = 0;
  await assert.rejects(
    waitForNpmTarball({
      name: "pkg",
      version: "1.0.0",
      timeoutMs: 50,
      now: () => {
        ticks += 1;
        return ticks === 1 ? 0 : 100;
      },
    }),
    /no attempt made for pkg@1.0.0/,
  );
});

test("runs the tarball waiter only when the module is executed directly", async () => {
  let started = false;
  runIfMain(module, module, () => {
    started = true;
  });
  assert.equal(started, true);
  runIfMain({}, module, () => {
    started = false;
  });
  assert.equal(started, true);
  await startFromCli(
    async () => {},
    () => {},
  );
  let caught;
  await startFromCli(
    async () => {
      throw new Error("cli failed");
    },
    (error) => {
      caught = error;
    },
  );
  assert.equal(caught.message, "cli failed");
});

test("readWithSignal rejects an already aborted signal", async () => {
  const signal = AbortSignal.abort(new Error("already aborted"));
  await assert.rejects(readWithSignal(Promise.resolve("ok"), signal), /already aborted/);
  const bare = new AbortController();
  bare.abort();
  await assert.rejects(readWithSignal(Promise.resolve("ok"), bare.signal), /aborted/);
});

test("readWithSignal falls back when abort has no reason", async () => {
  await assert.rejects(
    readWithSignal(Promise.resolve("ok"), {
      aborted: true,
      reason: undefined,
      addEventListener() {},
    }),
    /aborted/,
  );
  const listeners = [];
  const pending = readWithSignal(new Promise(() => {}), {
    aborted: false,
    reason: undefined,
    addEventListener(_type, listener) {
      listeners.push(listener);
    },
  });
  for (const listener of listeners) listener();
  await assert.rejects(pending, /aborted/);
});
