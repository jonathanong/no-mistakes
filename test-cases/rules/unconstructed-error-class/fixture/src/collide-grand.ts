import { B } from "./collide";

// Not flagged: `B.CollideBase` extends `Array`, and the cross-file `extends`
// resolves by class id to that class, not to the error class `A.CollideBase`
// that shares its scope.
export class CollideGrand extends B.CollideBase {}

// Not flagged: `B.BareBase` extends nothing, and the cross-file `extends`
// resolves by class id to that class, not to the error class `A.BareBase`.
export class BareGrand extends B.BareBase {}
