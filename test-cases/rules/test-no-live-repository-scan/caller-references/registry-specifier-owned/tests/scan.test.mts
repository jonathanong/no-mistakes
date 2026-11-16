import { workers } from "../src/safe.mts";
import { marker } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const root = mkdtempSync(join(tmpdir(), "scan-"));
void marker;
let name = "sync";
workers[name](root);
workers.sync(root);
