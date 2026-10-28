import { readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import extra, { readdir as extraRead } from "fs-extra";

// The shared temp directory and a predictable child are not owned test input.
readdirSync(tmpdir()); // shared-temp
readdirSync(join(tmpdir(), "shared-prefix")); // shared-child

// Fixture allowance intentionally matches a segment at any depth.
readdirSync(join(import.meta.dirname, ".github", "fixtures")); // nested-fixture
readdirSync(join(import.meta.dirname, ".github", "fixtures", "..", "workflows")); // escaped-fixture

import.meta.glob("../src/**/*.ts"); // meta-live
import.meta["glob"]("./fixtures/**/*.json"); // meta-fixture
import.meta.globEager("../workflows/**"); // meta-eager
import.meta.glob(process.env.INPUT); // meta-unresolved
import.meta.glob(["./fixtures/**", "../src/**"]); // meta-array-unresolved

extra.readdirSync(".github"); // extra-live
extra.readdirSync(join(import.meta.dirname, "fixtures")); // extra-fixture
await extraRead("src"); // extra-named

// A local API with the same method name is outside the imported enumerator set.
const local = { glob: (_pattern: string) => [] };
local.glob("src"); // local-glob
