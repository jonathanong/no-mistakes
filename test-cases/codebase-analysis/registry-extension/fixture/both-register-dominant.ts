import { A } from "./a";
import { B } from "./b";
import { C } from "./c";
import { registry } from "./registry";

// 3 register calls (dominant) plus a 2-entry container literal.
registry.register(new A());
registry.register(new B());
registry.register(new C());

export default [new A(), new B()];
