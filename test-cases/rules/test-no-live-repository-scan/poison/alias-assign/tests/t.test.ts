// Reported: aliasing process.env by assignment hands out a mutable alias.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
let alias;
alias = process.env;
alias.POISON_DIR = "./src";
list();
