import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { workers } from "@scope/pkg";

const root = mkdtempSync(join(tmpdir(), "scan-"));
let name = "sync";
workers[name](root);
