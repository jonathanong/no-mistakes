import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import "../src/entry.mts";
scan(".");
