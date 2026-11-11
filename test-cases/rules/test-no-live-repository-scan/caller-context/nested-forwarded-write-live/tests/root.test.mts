import { main } from "../src/main.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const root = mkdtempSync(join(tmpdir(), "nested-forwarded-"));
main({ nested: { root } });
