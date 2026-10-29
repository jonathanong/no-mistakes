import { createRequire as make } from "node:module";
const escaped = make(import.meta.url);
escape(escaped);
escaped("node:fs"); // escaped-loader
const namespace = require("node:module");
namespace.createRequire = replacement;
namespace.createRequire(__filename)("glob"); // mutated-namespace
let factory = make;
factory = replacement;
factory(__filename)("fs-extra"); // mutated-factory
module.require = replacement;
module.require("node:fs"); // mutated-module
