// Accepted: the owned write dominates both entry calls of a two-hop chain.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { chainA, chainB } from "../src/env-chain";

process.env.CHAIN_DIR = mkdtempSync(join(tmpdir(), "env-chain-"));
chainA();
chainB();
