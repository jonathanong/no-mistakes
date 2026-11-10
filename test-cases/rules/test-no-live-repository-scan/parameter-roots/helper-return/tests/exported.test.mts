import { cpSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { ensure, exported } from "../src/exported.mts";

const dirs: string[] = [];
const manifestPath = "manifest.json";

function mkTmp(prefix: string) {
  const directory = mkdtempSync(join(tmpdir(), prefix));
  dirs.push(directory);
  return directory;
}

function mkFixture() {
  const root = mkTmp("p-");
  const source = join(root, "c");
  cpSync("fixtures/x", source, { recursive: true });
  return { output: join(root, "o"), source };
}

const { output, source } = mkFixture();
exported(source, [manifestPath]);
ensure({ output, source });
for (const directory of dirs) rmSync(directory, { recursive: true });
