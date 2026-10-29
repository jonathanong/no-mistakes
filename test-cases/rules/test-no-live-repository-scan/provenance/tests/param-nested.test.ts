// Reported: the enumeration is in a callback, not in the called function.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { syncNested } from "../src/param-nested";

syncNested([mkdtempSync(join(tmpdir(), "param-nested-"))]);
