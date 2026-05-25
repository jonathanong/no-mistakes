const assert = require("node:assert/strict");
const { createServer } = require("node:http");
const { EventEmitter } = require("node:events");
const { mkdtemp, readFile, rm, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { tmpdir } = require("node:os");
const { pathToFileURL } = require("node:url");

const { download, fetchText, isRedirectStatus } = require("./install/index");
const { request } = require("./install/index");

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
