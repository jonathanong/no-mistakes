type NodeRequire = (specifier: string) => unknown;
declare const require: NodeRequire;
require("./ambient");
(require as NodeRequire)("./wrapped-ambient");
