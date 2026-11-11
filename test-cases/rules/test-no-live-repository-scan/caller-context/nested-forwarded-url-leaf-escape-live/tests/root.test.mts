import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { main } from "../src/main.mts";
const root = pathToFileURL(mkdtempSync(join(tmpdir(), "case-")));
main({ nested: { root } });
