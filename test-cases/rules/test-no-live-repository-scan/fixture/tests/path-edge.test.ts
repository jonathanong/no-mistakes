// Edge shapes of path provenance; none can be shown bounded, so each is reported.
import { readdirSync, globSync } from "node:fs";
import { dirname } from "node:path";
import { importedDir } from "../helpers/dirs";

declare const base: object;
declare const make: () => string;
let loose = {};
const { picked } = { picked: "x" };

readdirSync(new URL("x")); // URL without a base
readdirSync(dirname()); // dirname without an argument
readdirSync("a" + "b"); // binary expression
readdirSync((() => "x")()); // callee that is not a name
readdirSync(__dirname); // unbound CommonJS global
readdirSync(picked); // destructured binding
readdirSync(importedDir); // import binding
readdirSync(make()); // call of an unknown function
for (const each of ["a", "b"]) readdirSync(each); // declarator without init
globSync("*.yml", { ignore: ["x"] }); // options without cwd
globSync("*.yml", { ...base }); // spread options
globSync("*.yml", loose); // options in a mutable binding
