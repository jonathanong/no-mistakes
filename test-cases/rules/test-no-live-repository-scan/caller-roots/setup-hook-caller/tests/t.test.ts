import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/list";

list(mkdtempSync(join(tmpdir(), "owned-")));
