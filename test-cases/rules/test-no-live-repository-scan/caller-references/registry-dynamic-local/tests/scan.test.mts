import { mkdtempSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

function processSync(root: string) {
  return readdirSync(root);
}

const workers = { sync: processSync };
const root = mkdtempSync(join(tmpdir(), "scan-"));
let name = "sync";
workers[name](root);
