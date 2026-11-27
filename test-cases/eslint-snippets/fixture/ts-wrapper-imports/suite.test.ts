import a = require("./a.test");
export * from "./b.test";
export { a } from "./c.test";
(require as NodeRequire)("./d.test");
require("./e.test" as const);
const loaded = require("./f.test") as unknown;
