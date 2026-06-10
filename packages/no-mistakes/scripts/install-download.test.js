const assert = require("node:assert/strict");
const { createServer } = require("node:http");
const { EventEmitter } = require("node:events");
const { mkdtemp, readFile, rm, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { tmpdir } = require("node:os");
const { pathToFileURL } = require("node:url");

const {
  HttpError,
  computeBackoffMs,
  download,
  fetchText,
  isRedirectStatus,
  isRetryableError,
  isRetryableStatus,
  parsePositiveInt,
  request,
  withRetry,
} = require("./install/index");

test("downloads file URLs and fetches text over HTTP redirects", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-download-"));
  const source = join(root, "source.txt");
  const destination = join(root, "destination.txt");
  const fileUrl = pathToFileURL(source).toString().replace("file://", "file:");
  const alternateTextPath = join(root, "alternate.txt");
  const alternateSource = pathToFileURL(alternateTextPath).toString().replace("file://", "file:");
  await writeFile(source, "hello");
  await writeFile(alternateTextPath, "alt");

  const server = createServer((request, response) => {
    if (request.url === "/missing-location") {
      response.writeHead(302);
      response.end();
      return;
    }
    if (request.url === "/download") {
      response.writeHead(200);
      response.end("remote");
      return;
    }
    if (request.url === "/redirect") {
      response.writeHead(302, { location: "/text" });
      response.end();
      return;
    }
    if (request.url === "/text") {
      response.writeHead(200);
      response.end("redirected");
      return;
    }
    response.writeHead(404);
    response.end("not found");
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await download(pathToFileURL(source).toString(), destination);
    await download(fileUrl, destination);
    assert.equal(await readFile(destination, "utf8"), "hello");
    await download(`http://127.0.0.1:${address.port}/download`, destination);
    assert.equal(await readFile(destination, "utf8"), "remote");
    assert.equal(await fetchText(fileUrl), "hello");
    assert.equal(await fetchText(`http://127.0.0.1:${address.port}/redirect`), "redirected");
    assert.equal(await fetchText(alternateSource), "alt");
    await assert.rejects(() => fetchText(`http://127.0.0.1:${address.port}/missing`), /HTTP 404/);
    await assert.rejects(
      () => fetchText(`http://127.0.0.1:${address.port}/missing-location`),
      /missing Location/i,
    );
    await assert.rejects(() => fetchText(`https://127.0.0.1:${address.port}/text`));
  } finally {
    await new Promise((resolve) => server.close(resolve));
    await rm(root, { recursive: true, force: true });
  }
});

test("classifies redirect status codes", () => {
  assert.equal(isRedirectStatus(301), true);
  assert.equal(isRedirectStatus(undefined), false);
});

test("rejects redirect loops", async () => {
  const server = createServer((_request, response) => {
    response.writeHead(302, { location: "/loop" });
    response.end();
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await assert.rejects(
      () => fetchText(`http://127.0.0.1:${address.port}/loop`),
      /Too many redirects/,
    );
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("request rejects timeout errors", async () => {
  const client = {
    get() {
      const req = new EventEmitter();
      req.setTimeout = (_timeout, callback) => {
        queueMicrotask(callback);
      };
      req.destroy = (error) => {
        req.emit("error", error);
      };
      return req;
    },
  };

  await assert.rejects(
    () => request("http://example.test/file", () => {}, 0, { http: client, https: client }, 1),
    /timed out after 1ms/,
  );
});

test("treats malformed urls as non-file URLs", async () => {
  await assert.rejects(() => download("%", "noop.txt"), /ERR_INVALID_URL|Invalid URL/);
});

test("fetchText limits response size", async () => {
  const server = createServer((_request, response) => {
    response.writeHead(200);
    // Send 1MB + 1 byte
    response.write(Buffer.alloc(1024 * 1024));
    response.write(Buffer.alloc(1));
    response.end();
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await assert.rejects(
      () => fetchText(`http://127.0.0.1:${address.port}/large`),
      /exceeded maximum size/,
    );
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("download retries on transient 502 then succeeds", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-download-retry-"));
  const destination = join(root, "asset.bin");
  let hits = 0;
  const server = createServer((_request, response) => {
    hits += 1;
    if (hits < 3) {
      response.writeHead(502);
      response.end("bad gateway");
      return;
    }
    response.writeHead(200);
    response.end("ok-bytes");
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  const logger = { warn: () => {} };

  try {
    await download(`http://127.0.0.1:${address.port}/asset`, destination, 0, () => {}, {
      maxAttempts: 4,
      baseDelayMs: 1,
      delay: () => Promise.resolve(),
      logger,
    });
    assert.equal(await readFile(destination, "utf8"), "ok-bytes");
    assert.equal(hits, 3);
  } finally {
    await new Promise((resolve) => server.close(resolve));
    await rm(root, { recursive: true, force: true });
  }
});

test("download exhausts retries on persistent 503", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-download-retry-"));
  const destination = join(root, "asset.bin");
  let hits = 0;
  const server = createServer((_request, response) => {
    hits += 1;
    response.writeHead(503);
    response.end("unavailable");
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await assert.rejects(
      () =>
        download(`http://127.0.0.1:${address.port}/asset`, destination, 0, () => {}, {
          maxAttempts: 3,
          baseDelayMs: 1,
          delay: () => Promise.resolve(),
          logger: { warn: () => {} },
        }),
      /HTTP 503/,
    );
    assert.equal(hits, 3);
  } finally {
    await new Promise((resolve) => server.close(resolve));
    await rm(root, { recursive: true, force: true });
  }
});

test("download does not retry on 404", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-download-retry-"));
  const destination = join(root, "asset.bin");
  let hits = 0;
  const server = createServer((_request, response) => {
    hits += 1;
    response.writeHead(404);
    response.end("nope");
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await assert.rejects(
      () =>
        download(`http://127.0.0.1:${address.port}/asset`, destination, 0, () => {}, {
          maxAttempts: 5,
          baseDelayMs: 1,
          delay: () => Promise.resolve(),
          logger: { warn: () => {} },
        }),
      /HTTP 404/,
    );
    assert.equal(hits, 1);
  } finally {
    await new Promise((resolve) => server.close(resolve));
    await rm(root, { recursive: true, force: true });
  }
});

test("fetchText retries on transient 504 then succeeds", async () => {
  let hits = 0;
  const server = createServer((_request, response) => {
    hits += 1;
    if (hits < 2) {
      response.writeHead(504);
      response.end("gateway timeout");
      return;
    }
    response.writeHead(200);
    response.end("payload");
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    const text = await fetchText(`http://127.0.0.1:${address.port}/text`, () => {}, {
      maxAttempts: 3,
      baseDelayMs: 1,
      delay: () => Promise.resolve(),
      logger: { warn: () => {} },
    });
    assert.equal(text, "payload");
    assert.equal(hits, 2);
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("retries network errors via ECONNRESET", async () => {
  let calls = 0;
  const client = {
    get() {
      const req = new EventEmitter();
      req.setTimeout = () => {};
      req.destroy = () => {};
      calls += 1;
      queueMicrotask(() => {
        const err = new Error("socket hang up");
        err.code = "ECONNRESET";
        req.emit("error", err);
      });
      return req;
    },
  };

  let delayCalled = 0;
  const result = await withRetry(
    () => request("http://example.test/file", () => {}, 0, { http: client, https: client }, 100),
    {
      maxAttempts: 3,
      baseDelayMs: 1,
      delay: async () => {
        delayCalled += 1;
      },
      logger: { warn: () => {} },
    },
  ).catch((e) => e);

  assert.equal(calls, 3);
  assert.equal(delayCalled, 2);
  assert.match(result.message, /socket hang up/);
});

test("withRetry passes through on success without delay", async () => {
  let delayCalled = false;
  const result = await withRetry(async () => 42, {
    maxAttempts: 3,
    baseDelayMs: 1,
    delay: async () => {
      delayCalled = true;
    },
  });
  assert.equal(result, 42);
  assert.equal(delayCalled, false);
});

test("withRetry throws non-retryable error immediately", async () => {
  let calls = 0;
  await assert.rejects(
    () =>
      withRetry(
        async () => {
          calls += 1;
          throw new Error("fatal unretryable error");
        },
        {
          maxAttempts: 3,
          baseDelayMs: 1,
          delay: () => Promise.resolve(),
        },
      ),
    /fatal unretryable error/,
  );
  assert.equal(calls, 1);
});

test("withRetry maxAttempts:1 disables retry", async () => {
  let calls = 0;
  await assert.rejects(
    () =>
      withRetry(
        async () => {
          calls += 1;
          throw new HttpError("http://example.test/x", 502);
        },
        {
          maxAttempts: 1,
          baseDelayMs: 1,
          delay: () => Promise.resolve(),
        },
      ),
    /HTTP 502/,
  );
  assert.equal(calls, 1);
});

test("withRetry honors env defaults when option is undefined", async () => {
  const prevAttempts = process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS;
  const prevBase = process.env.NO_MISTAKES_DOWNLOAD_RETRY_BASE_MS;
  process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS = "2";
  process.env.NO_MISTAKES_DOWNLOAD_RETRY_BASE_MS = "1";
  let calls = 0;
  try {
    await assert.rejects(
      () =>
        withRetry(
          async () => {
            calls += 1;
            throw new HttpError("http://example.test/x", 502);
          },
          { delay: () => Promise.resolve(), logger: { warn: () => {} } },
        ),
      /HTTP 502/,
    );
    assert.equal(calls, 2);
  } finally {
    if (prevAttempts === undefined) delete process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS;
    else process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS = prevAttempts;
    if (prevBase === undefined) delete process.env.NO_MISTAKES_DOWNLOAD_RETRY_BASE_MS;
    else process.env.NO_MISTAKES_DOWNLOAD_RETRY_BASE_MS = prevBase;
  }
});

test("withRetry falls back to defaults on invalid env values", async () => {
  const prevAttempts = process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS;
  process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS = "not-a-number";
  let calls = 0;
  try {
    await assert.rejects(
      () =>
        withRetry(
          async () => {
            calls += 1;
            throw new HttpError("http://example.test/x", 502);
          },
          { baseDelayMs: 1, delay: () => Promise.resolve(), logger: { warn: () => {} } },
        ),
      /HTTP 502/,
    );
    // default DEFAULT_MAX_ATTEMPTS is 4
    assert.equal(calls, 4);
  } finally {
    if (prevAttempts === undefined) delete process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS;
    else process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS = prevAttempts;
  }
});

test("HttpError and classification helpers", () => {
  const err = new HttpError("http://example.test/x", 502);
  assert.equal(err.statusCode, 502);
  assert.equal(err.retryable, true);
  assert.equal(err.name, "HttpError");
  assert.match(err.message, /HTTP 502/);

  assert.equal(isRetryableStatus(500), true);
  assert.equal(isRetryableStatus(599), true);
  assert.equal(isRetryableStatus(408), true);
  assert.equal(isRetryableStatus(429), true);
  assert.equal(isRetryableStatus(404), false);
  assert.equal(isRetryableStatus(undefined), false);

  assert.equal(isRetryableError(null), false);
  assert.equal(isRetryableError(new Error("nope")), false);
  const e = new Error("net");
  e.code = "ECONNRESET";
  assert.equal(isRetryableError(e), true);
  const premature = new Error("Premature close");
  premature.code = "ERR_STREAM_PREMATURE_CLOSE";
  assert.equal(isRetryableError(premature), true);
  const tagged = new Error("explicit");
  tagged.retryable = true;
  assert.equal(isRetryableError(tagged), true);
});

test("parsePositiveInt rejects non-positive and non-integer", () => {
  assert.equal(parsePositiveInt(undefined, 7), 7);
  assert.equal(parsePositiveInt(null, 7), 7);
  assert.equal(parsePositiveInt("", 7), 7);
  assert.equal(parsePositiveInt("abc", 7), 7);
  assert.equal(parsePositiveInt("0", 7), 7);
  assert.equal(parsePositiveInt("-1", 7), 7);
  assert.equal(parsePositiveInt("1.5", 7), 7);
  assert.equal(parsePositiveInt("3", 7), 3);
  assert.equal(parsePositiveInt(5, 7), 5);
});

test("withRetry uses default sleep when delay option is omitted", async () => {
  let calls = 0;
  await assert.rejects(
    () =>
      withRetry(
        async () => {
          calls += 1;
          throw new HttpError("http://example.test/x", 502);
        },
        { maxAttempts: 2, baseDelayMs: 1, logger: { warn: () => {} } },
      ),
    /HTTP 502/,
  );
  assert.equal(calls, 2);
});

test("computeBackoffMs respects cap and jitter", () => {
  // attempt 1: base 500, exponential 500, capped at 4000, random=1 → 499
  assert.equal(
    computeBackoffMs(1, 500, () => 0.999),
    Math.floor(0.999 * 500),
  );
  // attempt 10: very large, capped at 4000
  assert.equal(
    computeBackoffMs(10, 500, () => 0.5),
    Math.floor(0.5 * 4000),
  );
  // random=0 → 0
  assert.equal(
    computeBackoffMs(3, 500, () => 0),
    0,
  );
});

test("rejects non-http/https redirect URLs (e.g. ftp:)", async () => {
  const server = createServer((request, response) => {
    if (request.url === "/redirect") {
      response.writeHead(302, { location: "ftp://example.com/file" });
      response.end();
      return;
    }
    response.writeHead(200);
    response.end("ok");
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await assert.rejects(
      () => fetchText(`http://127.0.0.1:${address.port}/redirect`),
      /Unsupported redirect protocol/,
    );
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("rejects invalid redirect Location headers", async () => {
  const server = createServer((request, response) => {
    if (request.url === "/redirect") {
      response.writeHead(302, { location: "http://[invalid-url" });
      response.end();
      return;
    }
    response.writeHead(200);
    response.end("ok");
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    await assert.rejects(
      () => fetchText(`http://127.0.0.1:${address.port}/redirect`),
      /Invalid redirect Location header/,
    );
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("validates redirected URLs before following", async () => {
  const server = createServer((request, response) => {
    if (request.url === "/redirect") {
      response.writeHead(302, { location: "http://bad-target.localhost/example" });
      response.end();
      return;
    }
    response.writeHead(200);
    response.end("ok");
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();

  try {
    const validateUrl = (url) => {
      if (url.includes("bad-target.localhost")) {
        throw new Error("disallowed redirect host");
      }
    };

    await assert.rejects(
      () => fetchText(`http://127.0.0.1:${address.port}/redirect`, validateUrl),
      /disallowed redirect host/,
    );
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});
