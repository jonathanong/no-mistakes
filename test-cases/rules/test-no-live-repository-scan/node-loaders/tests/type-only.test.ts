import type { createRequire } from "node:module";
import type * as nodeModule from "module";
// Erased imports do not establish a runtime Node factory identity.
createRequire("url")("node:fs");
nodeModule.createRequire("url")("node:fs");
