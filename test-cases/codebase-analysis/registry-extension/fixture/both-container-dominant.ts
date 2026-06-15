import { A } from "./a";
import { B } from "./b";
import { C } from "./c";
import { registry } from "./registry";

// 2 register calls plus a 3-entry container literal (dominant).
registry.register(new A());
registry.register(new B());

export default [new A(), new B(), new C()];
