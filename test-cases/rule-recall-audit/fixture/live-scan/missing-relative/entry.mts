import * as absent from "./missing.mjs";
import {scan} from "./helper.mjs";
import {mkdtempSync} from "node:fs";
import {tmpdir} from "node:os";
import {join} from "node:path";
scan(mkdtempSync(join(tmpdir(), "audit-")));
