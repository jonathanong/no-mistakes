import { A } from "./a";
import { B } from "./b";
import { alpha } from "./alpha-registry";
import { beta } from "./beta-registry";

// Two different registries that share the `register` method name must NOT be
// collapsed into one repeated pattern.
alpha.register(new A());
beta.register(new B());
