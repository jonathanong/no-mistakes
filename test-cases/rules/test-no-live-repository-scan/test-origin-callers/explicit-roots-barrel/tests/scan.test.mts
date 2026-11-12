import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { scan } from "../src/scan.mts";
import * as barrel from "../src/barrel.mts";
const root = mkdtempSync(join(tmpdir(), "owned-"));
scan(root);
export const escaped = barrel;
