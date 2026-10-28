// The second load must remain visible when the first binding resolves.
const fs = require("node:fs");
fs.readdirSync("packages");
const { readdirSync: other } = require("node:fs");
other("docs");

// A non-enumerating destructured export cannot create another scan.
const { readFileSync, readdirSync } = require("node:fs");
readdirSync("src");
readFileSync("fixtures/a.txt");
const { readFileSync: readOnly } = require("node:fs");
readOnly("fixtures/a.txt");

// This parameter shadows the loader and must not be treated as a module load.
function withShadow(require: (name: string) => { readdirSync(path: string): string[] }) {
  const shadow = require("node:fs");
  return shadow.readdirSync("src");
}
void withShadow;

// Suppression applies to the require declaration itself.
// no-mistakes-disable-next-line test-no-live-repository-scan
let { readdirSync: muted } = require("node:fs");
muted("docs");

// The member call and load start at the same source offset: one advisory suffices.
require("node:fs").readdirSync("src");
