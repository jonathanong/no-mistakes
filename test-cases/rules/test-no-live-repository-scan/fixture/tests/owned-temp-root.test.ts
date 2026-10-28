// Negative: the test creates and owns the directory it enumerates.
import { mkdtempSync, readdirSync, writeFileSync } from "node:fs";
import { mkdtemp, readdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const root = mkdtempSync(join(tmpdir(), "scan-"));
writeFileSync(join(root, "a.txt"), "a");
readdirSync(root);
readdirSync(join(root, "nested"));

const later = await mkdtemp(join(tmpdir(), "later-"));
await readdir(later);
