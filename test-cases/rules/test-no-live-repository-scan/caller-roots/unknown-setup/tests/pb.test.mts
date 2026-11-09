import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pb } from "../src/pb.mts";

pb(mkdtempSync(join(tmpdir(), "p-")));
