const { createRequire: make } = require("node:module");
const nodeModule = require("module");
make(__filename)("node:fs"); // commonjs-named-factory
nodeModule.createRequire(__filename)("glob"); // commonjs-namespace-factory
require("node:module").createRequire(__filename)("fs-extra"); // commonjs-direct-factory
