import { A } from "./a";
import { B } from "./b";
import { alpha } from "./alpha-registry";
import { beta } from "./beta-registry";

// Two registries with equal entry counts: the winner must be deterministic.
alpha.register(new A());
alpha.register(new A());
beta.register(new B());
beta.register(new B());
