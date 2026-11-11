import { mkdtempSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { it } from "vitest";
it.concurrent("unrelated", () => { readdirSync(mkdtempSync(join(tmpdir(), "other-"))); });
