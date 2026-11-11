import { scan } from "../src/scan.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const module = await import("../src/index.mts");
const selected = module.other;
void selected;
scan(".");
