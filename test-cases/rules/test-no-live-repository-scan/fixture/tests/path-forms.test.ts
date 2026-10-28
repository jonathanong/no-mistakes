// Path provenance forms. Lines marked LIVE or UNRESOLVED are asserted by the CLI test.
import { readdirSync, globSync, mkdtempSync, realpathSync } from "node:fs";
import { tmpdir } from "node:os";
import path, { dirname, join, posix } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const viaUrl = fileURLToPath(new URL("../src", import.meta.url));
const owned = mkdtempSync(`${tmpdir()}/policy-`);
let mutable = "src";
const param = (dir: string) => dir;

readdirSync(join(here, "..", "src")); // LIVE module-relative
readdirSync(viaUrl); // LIVE URL anchored at the module
readdirSync(`${here}/src`); // LIVE template
readdirSync(realpathSync(path.resolve("."))); // LIVE relative
readdirSync(posix.normalize("a/b")); // LIVE relative
readdirSync(join(owned, "nested")); // allowed temp root
readdirSync(`${tmpdir()}/x`); // allowed temp root
readdirSync(join(here, "__fixtures__", "a")); // allowed fixture segment
globSync("*.yml", { cwd: here }); // LIVE glob cwd
globSync("*.yml", { cwd: join(here, "fixtures") }); // allowed glob cwd
readdirSync(mutable); // UNRESOLVED mutable binding
readdirSync(param("x")); // UNRESOLVED call not modeled
readdirSync(...[here]); // UNRESOLVED spread
readdirSync(process.env.ROOT as string); // UNRESOLVED environment
readdirSync(await Promise.resolve(here)); // UNRESOLVED await of a call
