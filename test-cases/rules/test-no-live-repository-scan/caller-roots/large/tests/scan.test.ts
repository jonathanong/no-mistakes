import * as fs from "node:fs";
import { mkdtempSync } from "node:fs";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
import { owned0, unknown0 } from "../src/long-chain";
import {
  scan,
  scanEnv,
  scanForwarded,
  scanLiteral,
  scanMixed,
  scanMutable,
  scanOptions,
  scanUnknown,
} from "../src/scan";

const moduleRoot = mkdtempSync(join(tmpdir(), "scan-module-"));
const fixtureRoot = join(import.meta.dirname, "../samples");

scan(mkdtempSync(join(tmpdir(), "scan-inline-"))); // inline-temp
const asyncRoot = await mkdtemp(join(tmpdir(), "scan-async-"));
scan(asyncRoot); // async-temp
function scanLocalRoot() {
  const localRoot = mkdtempSync(join(tmpdir(), "scan-local-"));
  scan(localRoot); // local-temp
}
scanLocalRoot();
scan(moduleRoot); // module-temp
scan(fixtureRoot); // configured-fixture
scanOptions({ root: mkdtempSync(join(tmpdir(), "scan-options-")) }); // options-field
const envRoot = mkdtempSync(join(tmpdir(), "scan-env-"));
vi.stubEnv("SCAN_ROOT", envRoot);
scanEnv(); // stubbed-env
vi.stubEnv("SCAN_ROOT", fixtureRoot);
scanEnv(); // stubbed-fixture-env
const original = fs.readdirSync;
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args)); // raw-spread-forwarder
scanForwarded(mkdtempSync(join(tmpdir(), "scan-forward-"))); // spread-forwarding-spy

scanLiteral("/workspace/source"); // unowned-literal
scanMixed(mkdtempSync(join(tmpdir(), "scan-mixed-owned-"))); // mixed-owned-caller
scanMixed("/workspace/source"); // mixed-live-caller
scanUnknown(unresolvedRoot); // unresolved-caller
let reassignedRoot = mkdtempSync(join(tmpdir(), "scan-let-"));
reassignedRoot = "/workspace/source";
scanMutable(reassignedRoot); // let-reassignment

scan(mkdtempSync(join(tmpdir(), "scan-extra-"))); // extra-owned-caller
owned0(mkdtempSync(join(tmpdir(), "owned-chain-"))); // owned-long-chain
unknown0(unresolvedChainRoot); // unresolved-long-chain
