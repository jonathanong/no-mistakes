import { list } from "./src/list";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

list(mkdtempSync(join(tmpdir(), "owned-")));
