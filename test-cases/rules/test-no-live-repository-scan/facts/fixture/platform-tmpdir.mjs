import { readdirSync } from "node:fs";
import { tmpdir } from "node:os";
readdirSync(tmpdir());
readdirSync(process.env.TMPDIR);
readdirSync(process.env.TEMP);
