import { A } from "./a";
import { B } from "./b";
import { registry } from "./registry";

// Register calls inside a default-exported setup function must still be found.
export default function setup() {
  registry.register(new A());
  registry.register(new B());
}
