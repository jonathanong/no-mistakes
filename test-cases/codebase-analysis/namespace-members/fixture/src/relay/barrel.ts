import Pivot from "./hub";
import { Spare } from "./hub";

// A named re-export, a renamed one, an import that is exported again, and a
// default import exported under another name.
export { Hub, Other as Renamed } from "./hub";
export { Spare };
export { Pivot as Hinge };

// Neither of these is a namespace of this repository.
export const plain = { Thing: class {} };
export { Package } from "some-package";
