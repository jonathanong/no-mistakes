// Reported: one caller passes an owned directory, but the other passes a live one.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { syncMixed } from "../src/param-mixed";

syncMixed("a", mkdtempSync(join(tmpdir(), "param-mixed-")));
syncMixed("b", join(import.meta.dirname, "..", "src"));
