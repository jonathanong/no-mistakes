import vt from "vitest";
vt.test.sequential("namespaced", () => {});
import vitest from "vitest";
vitest.sequential("default import is not a runner", () => {});
import vt2 = require("vitest");
vt2.test.sequential("import equals", () => {});
