import { B } from "./collide";

// Not flagged: `B.CollideBase` extends `Array`, and a cross-file `extends` of a
// scope that two declarations share does not resolve to the error class in `A`.
export class CollideGrand extends B.CollideBase {}
