import vt from "vitest";
import path from "node:path";
vt.test.skip("namespace member", () => {});
vt.skip("direct default", () => {});
path.skip("other default", () => {});
