import { z } from "some-package";
import Fallback from "./fallback";
import { copiedStar } from "./copied-star";
import { plain } from "./barrel";

declare function register(value: unknown): void;

// Each import is read as a value. A default import of a namespace and a module
// the chain cannot follow may hold a namespace, so their classes may be built
// through the copy. A package outside the repository and a plain object hold
// none.
register(z);
register(Fallback);
register(copiedStar);
register(plain);
