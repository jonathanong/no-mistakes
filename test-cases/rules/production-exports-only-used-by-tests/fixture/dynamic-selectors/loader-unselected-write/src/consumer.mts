const registry = { x: () => import("./worker.mjs"), y: () => import("./other.mjs") };
registry.x = otherLoader;
await registry.y();
