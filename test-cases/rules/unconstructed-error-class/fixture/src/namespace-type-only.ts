// Erased at run time, so it is no use of the module: the class of
// `namespace-type-only-target.ts` is still reported.
import type Target = require("./namespace-type-only-target");

export type Alias = Target.TypeOnly.TypeOnlyError;
