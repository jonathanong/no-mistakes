import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { main } from "../src/main.mts";
main({ nested: { root: mkdtempSync(join(tmpdir(), "string-")) } });
