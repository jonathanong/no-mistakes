import {} from "./empty";
import { Foo } from "./foo";
import { Bar } from "./bar";
import { registry } from "./registry";

registry.register(new Foo());
registry.register(new Bar());
registry.register(localThing);
registry.register(42);
registry.register(new outer.inner.Thing());
registry.register(new (getCtor())());
(() => {})();

export default registry;
