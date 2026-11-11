import { writeResources } from "../src/writer.mts";
import { main } from "../src/main.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const root = mkdtempSync(join(tmpdir(), "context-"));
main({ outputDir: root, schemaDir: join(root, "schemas") });
