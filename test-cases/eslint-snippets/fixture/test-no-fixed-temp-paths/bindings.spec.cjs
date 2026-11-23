const { join: combine } = require("path");
const os = require("node:os");
const { tmpdir: temporaryRoot } = os;
const { mkdtempSync: makeDirectory } = require("node:fs");
const fs = require("fs");
const promiseFs = require("node:fs/promises");
const joinAlias = combine;
const createAlias = makeDirectory;

combine(temporaryRoot(), "fixed.json");
joinAlias(os["tmpdir"](), "fixed-again.json");
require("node:path").join(require("os").tmpdir(), "direct.json");

createAlias(combine(temporaryRoot(), "example-"));
fs.promises.mkdtemp(combine(temporaryRoot(), "example-"));
promiseFs.mkdtemp("/var/tmp/example-");
makeDirectory("/tmp/example-", "/tmp/invalid-option");

function shadowed(makeDirectory, combine, os, require, fs) {
  makeDirectory("/tmp/shadowed.json");
  fs.mkdtempSync("/tmp/fake-member.json");
  combine(temporaryRoot(), "shadowed-join.json");
  joinAlias(os.tmpdir(), "shadowed-root.json");
  require("node:path").join(temporaryRoot(), "shadowed-require.json");
  require("node:fs").mkdtempSync("/tmp/shadowed-require.json");
}

const unrelated = { join() {}, tmpdir() {}, mkdtemp() {} };
unrelated.join(temporaryRoot(), "unrelated-join.json");
joinAlias(unrelated.tmpdir(), "unrelated-root.json");
unrelated.mkdtemp(joinAlias(temporaryRoot(), "still-fixed.json"));
