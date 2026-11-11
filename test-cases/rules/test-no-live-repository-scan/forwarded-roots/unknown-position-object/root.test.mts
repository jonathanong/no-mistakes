import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "./list.mts";
const root = mkdtempSync(join(tmpdir(), "forwarded-"));
list(...unknownInputs, { outputRoot: root });
