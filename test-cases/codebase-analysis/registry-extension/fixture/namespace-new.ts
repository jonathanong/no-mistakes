import * as plugins from "./plugins";
import { registry } from "./registry";

// Namespace-import constructors (`new plugins.Foo()`).
registry.register(new plugins.Foo());
registry.register(new plugins.Bar());
