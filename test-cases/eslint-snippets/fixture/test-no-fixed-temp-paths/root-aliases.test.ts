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

function readFixture(root: string, segment: string) {
  readFileSync(join(root, "result.json"));
  readFileSync(join(osRoot, segment));
}

const prefix = join(osRoot, "example-");
mkdtempSync(prefix);
