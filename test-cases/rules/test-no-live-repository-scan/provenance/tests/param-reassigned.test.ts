// Reported: the callee rebinds the parameter before enumerating it.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { syncReassigned } from "../src/param-reassigned";

syncReassigned(mkdtempSync(join(tmpdir(), "param-reassigned-")));
