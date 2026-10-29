import { createRequire as make } from "node:module";
import * as nodeModule from "module";
import nodeDefault from "node:module";

make(import.meta.url)("node:fs"); // direct-factory
make(import.meta.url)(moduleName); // computed-factory
nodeModule.createRequire(import.meta.url)("glob"); // namespace-factory
nodeDefault["createRequire"](import.meta.url)("fast-glob"); // default-factory
const load = make(import.meta.url);
load("node:fs"); // bound-loader
const alias = load;
alias("fs-extra"); // loader-alias
const makeAlias = make;
makeAlias(import.meta.url)("globby"); // factory-alias
let mutable = make(import.meta.url);
mutable = otherLoader;
mutable("node:fs"); // mutated-loader
load("node:path"); // unrelated-factory-module

function custom(make: (path: string) => (name: string) => unknown) {
  make("url")("node:fs"); // shadowed-factory
}
custom(() => () => ({}));
