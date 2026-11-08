const registry = { x: () => import("./worker.mjs"), y: () => import("./other.mjs") };
consume(registry);
await registry.y();
