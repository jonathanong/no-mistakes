import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { runSmoke } from "../src/scan.mts";

const root = mkdtempSync(join(tmpdir(), "smoke-"));
runSmoke(root);
declare function register(callback: () => unknown): void;
import("../src/scan.mts").then((module) => register(module.runSmoke));
