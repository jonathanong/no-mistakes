// Reported: the argument is owned, but the callee climbs out of it with `..`.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { listTwoUp } from "../src/param-climb";

listTwoUp(mkdtempSync(join(tmpdir(), "param-climb-")));
