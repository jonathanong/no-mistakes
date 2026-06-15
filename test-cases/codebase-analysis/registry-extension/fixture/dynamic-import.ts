import { registry } from "./registry";

registry.register(() => import("./plugins/lazy-a"));
registry.register(() => import("./plugins/lazy-b"));
