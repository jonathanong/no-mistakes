import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const root = "/tmp/example" as const;
const systemRoot = "/var/tmp";
const osRoot = tmpdir();
const file = resolve(systemRoot, "result.json");
readFileSync(join(root, "result.json"));
readFileSync(file);
readFileSync(join(osRoot, "result.json"));

readFileSync(join(osRoot));
readFileSync(resolve(osRoot, "", ``));
readFileSync(resolve(osRoot, "/fixtures/result.json"));
readFileSync(resolve(systemRoot, "..", "fixtures", "result.json"));
const secondAlias = root;
readFileSync(join(secondAlias, "result.json"));
const base = osRoot;
const thirdAlias = base;
readFileSync(resolve(thirdAlias, "third-alias.json"));

function readFixture(root: string, segment: string) {
  const localRoot = root;
  readFileSync(join(localRoot, "result.json"));
  readFileSync(join(secondAlias, "outer-root.json"));
  readFileSync(join(osRoot, segment));
}

let mutableRoot = "/tmp";
const mutableAlias = mutableRoot;
mutableRoot = "/fixtures";
readFileSync(join(mutableAlias, "mutable-root.json"));

const reassignedRoot = "/tmp";
const reassignedAlias = reassignedRoot;
reassignedRoot = "/fixtures";
readFileSync(join(reassignedAlias, "reassigned-root.json"));

const cycleStart = cycleEnd;
const cycleEnd = cycleStart;
readFileSync(join(cycleEnd, "cycle.json"));

const prefix = join(osRoot, "example-");
mkdtempSync(prefix);
