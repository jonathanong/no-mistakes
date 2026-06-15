import "./plugins/side-effect-a";
import { FooPlugin } from "./plugins/foo";
import { registry } from "./registry";

// Dominant shape is register-call; the side-effect import is noted separately.
registry.register(new FooPlugin({ id: "one" }));
registry.register(new FooPlugin({ id: "two" }));
