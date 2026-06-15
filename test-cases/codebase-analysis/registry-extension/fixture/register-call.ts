import { FooPlugin } from "./plugins/foo";
import { BarPlugin } from "./plugins/bar";
import { registry } from "./registry";

registry.register(new FooPlugin({ id: "foo" }));
registry.register(new BarPlugin({ id: "bar" }));
