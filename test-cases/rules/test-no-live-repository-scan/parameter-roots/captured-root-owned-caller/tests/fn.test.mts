import { update } from "../src/update.mts";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

update(mkdtempSync(join(tmpdir(), "owned-")));

// File-wide consumer control: docs/rules/test-no-live-repository-scan.md.
const gate = { update: 1 };
const { update: hidden } = gate;
void hidden;
